use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde_json::json;

use crate::config::SpawnConfig;
use crate::expect::{ExpectLocator, ExpectScreen};
use crate::input::{self, Key, MouseButton};
use crate::locator::{Locator, Selector};
use crate::record::{GifOptions, GifRecorder};
use crate::render::{RenderOptions, TerminalRenderer, Theme};
use crate::screen::ScreenSnapshot;
use crate::trace::TraceWriter;

struct SharedState {
    parser: vt100::Parser,
    last_output_at: Instant,
}

/// A live PTY-backed terminal session — the Tuiwright equivalent of Playwright's `Page`.
///
/// Provides methods for sending input, querying screen state, creating locators,
/// making assertions, capturing screenshots, recording GIFs, and writing traces.
pub struct Page {
    config: SpawnConfig,
    started_at: Instant,
    state: Arc<Mutex<SharedState>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    #[allow(dead_code)]
    master: Box<dyn MasterPty + Send>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    theme: Theme,
    renderer: TerminalRenderer,
    recorder: Mutex<Option<GifRecorder>>,
    trace: Option<TraceWriter>,
}

impl Page {
    /// Spawn a new TUI session with the given configuration.
    pub fn spawn(config: SpawnConfig) -> Result<Self> {
        if config.program.trim().is_empty() {
            bail!("SpawnConfig.program cannot be empty");
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: config.size.rows,
                cols: config.size.cols,
                pixel_width: config.size.pixel_width,
                pixel_height: config.size.pixel_height,
            })
            .context("opening PTY")?;

        let mut cmd = CommandBuilder::new(&config.program);
        for arg in &config.args {
            cmd.arg(arg);
        }
        for (key, value) in &config.env {
            cmd.env(key, value);
        }
        if let Some(cwd) = &config.cwd {
            cmd.cwd(cwd);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .context("spawning child process")?;
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .context("cloning PTY reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("taking PTY writer")?;

        let theme = Theme::xterm_dark();
        let renderer = TerminalRenderer::new(RenderOptions::ci(), theme.clone());

        let parser = vt100::Parser::new(config.size.rows, config.size.cols, config.scrollback);

        let state = Arc::new(Mutex::new(SharedState {
            parser,
            last_output_at: Instant::now(),
        }));

        // Set up trace writer if requested
        let trace = if let Some(ref trace_path) = config.trace_path {
            Some(TraceWriter::new(trace_path)?)
        } else {
            None
        };

        if let Some(ref tw) = trace {
            tw.event(
                "spawn",
                json!({
                    "program": config.program,
                    "args": config.args,
                    "cols": config.size.cols,
                    "rows": config.size.rows,
                }),
            )?;
        }

        // Start background reader thread
        let state_clone = Arc::clone(&state);
        let trace_clone = trace.clone();
        thread::spawn(move || {
            reader_loop(reader, state_clone, trace_clone);
        });

        // Start recording if requested
        let recorder = if config.record {
            Mutex::new(Some(GifRecorder::new()))
        } else {
            Mutex::new(None)
        };

        Ok(Self {
            config,
            started_at: Instant::now(),
            state,
            writer: Arc::new(Mutex::new(writer)),
            master: pair.master,
            child: Mutex::new(child),
            theme,
            renderer,
            recorder,
            trace,
        })
    }

    /// Get the current screen snapshot.
    pub fn screen(&self) -> ScreenSnapshot {
        let state = self.state.lock().expect("state mutex poisoned");
        ScreenSnapshot::from_vt(state.parser.screen(), &self.theme)
    }

    /// Get the elapsed time since session start.
    pub fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
    }

    // ── Input actions ──────────────────────────────────────────────────

    /// Send a key press to the application.
    pub fn press(&self, key: Key) -> Result<()> {
        let app_cursor = {
            let state = self.state.lock().expect("state mutex poisoned");
            state.parser.screen().application_cursor()
        };
        let bytes = input::encode_key(key, app_cursor);
        self.write_pty(&bytes)?;

        if let Some(ref tw) = self.trace {
            tw.event("action", json!({"kind": "press", "key": format!("{key}")}))?;
        }

        self.maybe_record_frame();
        // Small delay for the app to process input
        thread::sleep(Duration::from_millis(10));
        Ok(())
    }

    /// Type a string of text into the application.
    pub fn type_text(&self, text: &str) -> Result<()> {
        let bytes = input::encode_text(text);
        self.write_pty(&bytes)?;

        if let Some(ref tw) = self.trace {
            tw.event("action", json!({"kind": "type_text", "text": text}))?;
        }

        self.maybe_record_frame();
        thread::sleep(Duration::from_millis(10));
        Ok(())
    }

    /// Paste text using bracketed paste if the app has enabled it.
    pub fn paste(&self, text: &str) -> Result<()> {
        let bracketed = {
            let state = self.state.lock().expect("state mutex poisoned");
            state.parser.screen().bracketed_paste()
        };
        let bytes = input::encode_paste(text, bracketed);
        self.write_pty(&bytes)?;

        if let Some(ref tw) = self.trace {
            tw.event("action", json!({"kind": "paste", "length": text.len()}))?;
        }

        self.maybe_record_frame();
        thread::sleep(Duration::from_millis(10));
        Ok(())
    }

    /// Click a cell at the given coordinates using SGR mouse protocol.
    pub fn click_cell(&self, col: u16, row: u16) -> Result<()> {
        let press = input::encode_sgr_mouse(MouseButton::Left, col, row, false);
        let release = input::encode_sgr_mouse(MouseButton::Left, col, row, true);
        self.write_pty(&press)?;
        thread::sleep(Duration::from_millis(5));
        self.write_pty(&release)?;

        if let Some(ref tw) = self.trace {
            tw.event("action", json!({"kind": "click", "col": col, "row": row}))?;
        }

        self.maybe_record_frame();
        thread::sleep(Duration::from_millis(10));
        Ok(())
    }

    /// Resize the terminal.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: cols.saturating_mul(10),
                pixel_height: rows.saturating_mul(20),
            })
            .context("resizing PTY")?;

        {
            let mut state = self.state.lock().expect("state mutex poisoned");
            state.parser.screen_mut().set_size(rows, cols);
        }

        if let Some(ref tw) = self.trace {
            tw.event("action", json!({"kind": "resize", "cols": cols, "rows": rows}))?;
        }

        // Wait for the app to redraw after resize
        thread::sleep(Duration::from_millis(50));
        self.maybe_record_frame();
        Ok(())
    }

    // ── Locators ───────────────────────────────────────────────────────

    /// Create a text locator for the given substring.
    pub fn get_by_text(&self, text: &str) -> Locator {
        Locator::new(Selector::Text(text.to_string()))
    }

    /// Create a regex locator.
    pub fn get_by_regex(&self, pattern: &str) -> Locator {
        Locator::new(Selector::Regex(pattern.to_string()))
    }

    /// Create a locator for a specific cell.
    pub fn get_by_cell(&self, row: u16, col: u16) -> Locator {
        Locator::new(Selector::Cell(row, col))
    }

    /// Create a locator for the cursor position.
    pub fn cursor(&self) -> Locator {
        Locator::new(Selector::CursorPosition)
    }

    // ── Waits ──────────────────────────────────────────────────────────

    /// Wait until the given text appears on screen.
    pub fn wait_for_text(&self, text: &str, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            let screen = self.screen();
            if screen.contains_text(text) {
                return Ok(());
            }
            if Instant::now() >= deadline {
                let screen_text = screen.plain_text();
                bail!(
                    "Timed out after {timeout:?} waiting for text {text:?}\n\nLast screen:\n{screen_text}"
                );
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Wait until the given regex matches on screen.
    pub fn wait_for_regex(&self, pattern: &str, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            let screen = self.screen();
            if screen.matches_regex(pattern) {
                return Ok(());
            }
            if Instant::now() >= deadline {
                let screen_text = screen.plain_text();
                bail!(
                    "Timed out after {timeout:?} waiting for regex {pattern:?}\n\nLast screen:\n{screen_text}"
                );
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Wait until no terminal output arrives for the given duration.
    pub fn wait_until_idle(&self, quiet_for: Duration) -> Result<()> {
        let timeout = self.config.default_timeout;
        let deadline = Instant::now() + timeout;
        loop {
            let last_output = {
                let state = self.state.lock().expect("state mutex poisoned");
                state.last_output_at
            };
            if last_output.elapsed() >= quiet_for {
                return Ok(());
            }
            if Instant::now() >= deadline {
                bail!("Timed out after {timeout:?} waiting for idle");
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    // ── Assertions ─────────────────────────────────────────────────────

    /// Create a screen-level expectation builder.
    pub fn expect_screen(&self) -> ExpectScreen<'_> {
        let timeout = self.config.default_timeout;
        ExpectScreen::new(move || self.screen(), timeout)
    }

    /// Create a locator-level expectation builder.
    pub fn expect_locator<'a>(&'a self, locator: &'a Locator) -> ExpectLocator<'a> {
        let timeout = self.config.default_timeout;
        ExpectLocator::new(locator, move || self.screen(), timeout)
    }

    // ── Artifacts ──────────────────────────────────────────────────────

    /// Capture a PNG screenshot of the current terminal state.
    pub fn screenshot(&self, path: impl AsRef<Path>) -> Result<()> {
        let screen = self.screen();
        self.renderer.save_screenshot(&screen, path.as_ref())?;

        if let Some(ref tw) = self.trace {
            tw.event(
                "screenshot",
                json!({"path": path.as_ref().display().to_string()}),
            )?;
        }

        Ok(())
    }

    /// Start recording frames for a GIF.
    pub fn start_recording(&self) -> Result<()> {
        let mut recorder = self.recorder.lock().expect("recorder mutex poisoned");
        *recorder = Some(GifRecorder::new());
        // Capture initial frame
        let screen = self.screen();
        if let Some(ref mut rec) = *recorder {
            rec.capture_frame(screen, self.elapsed_ms());
        }
        Ok(())
    }

    /// Stop recording and encode the accumulated frames as a GIF.
    pub fn stop_recording_gif(
        &self,
        path: impl AsRef<Path>,
        options: GifOptions,
    ) -> Result<()> {
        let mut recorder_guard = self.recorder.lock().expect("recorder mutex poisoned");
        let recorder = recorder_guard
            .take()
            .context("recording was not started")?;

        // Capture final frame
        // (already captured via maybe_record_frame during actions)

        recorder.encode_gif(path.as_ref(), &self.renderer, &options)?;

        if let Some(ref tw) = self.trace {
            tw.event(
                "recording",
                json!({
                    "path": path.as_ref().display().to_string(),
                    "frames": recorder.frame_count(),
                }),
            )?;
        }

        Ok(())
    }

    /// Save the accumulated trace to a JSONL file.
    pub fn save_trace(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(ref tw) = self.trace {
            tw.event("trace_saved", json!({"path": path.as_ref().display().to_string()}))?;
        }
        // Trace is already being written to the configured path.
        // This method is a no-op confirmation if trace was already configured.
        Ok(())
    }

    /// Kill the child process.
    pub fn kill(&self) -> Result<()> {
        let mut child = self.child.lock().expect("child mutex poisoned");
        child.kill().ok();
        Ok(())
    }

    // ── Internal helpers ───────────────────────────────────────────────

    fn write_pty(&self, bytes: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().expect("writer mutex poisoned");
        writer.write_all(bytes).context("writing to PTY")?;
        writer.flush().context("flushing PTY")?;
        Ok(())
    }

    fn maybe_record_frame(&self) {
        if let Ok(mut guard) = self.recorder.lock() {
            if let Some(ref mut recorder) = *guard {
                // Small delay to let screen update
                thread::sleep(Duration::from_millis(30));
                let screen = self.screen();
                recorder.capture_frame(screen, self.elapsed_ms());
            }
        }
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            child.kill().ok();
        }
    }
}

/// Background thread that continuously reads PTY output and feeds the parser.
fn reader_loop(
    mut reader: Box<dyn Read + Send>,
    state: Arc<Mutex<SharedState>>,
    trace: Option<TraceWriter>,
) {
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,  // EOF — child exited
            Ok(n) => {
                let chunk = &buf[..n];
                let mut state = state.lock().expect("state mutex poisoned");
                state.parser.process(chunk);
                state.last_output_at = Instant::now();

                if let Some(ref tw) = trace {
                    tw.event("output", json!({"bytes": n})).ok();
                }
            }
            Err(e) => {
                // EAGAIN / interrupted can happen; other errors mean the PTY is gone
                if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                break;
            }
        }
    }
}
