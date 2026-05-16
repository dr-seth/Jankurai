// Synthetic Rust stub for the deeper-AST (syn) re-derivation tests
// (ARY-2029). Line numbers below are load-bearing for the fixture docs.

pub struct AryaEngine;

impl AryaEngine {
    /// A real method — claimed and verified by both the regex resolver
    /// and the syn AST cross-check (positive control, line 9).
    pub fn sense(&self) -> u8 {
        0
    }

    pub fn decide(&self) -> u8 {
        // The string literal below CONTAINS a fake fn header.
        // PY_RUST_SYMBOL_RE matches "fn ghost_decide" on line 20, but
        // syn parses lines 19-21 as a string literal, not an item —
        // so ghost_decide is NOT in the AST set. This is the
        // regex-spoof the deeper-AST check must reject.
        let _spoof = "
fn ghost_decide(&self) {}
";
        1
    }
}

// A genuine top-level fn at a known line for the positive control.
fn real_top_level() -> bool {
    true
}
