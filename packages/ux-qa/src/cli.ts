#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { chromium } from "playwright";
import type { Page } from "playwright";
import { analyzePage } from "./page-analyzer.js";
import type { UxQaArtifact, UxQaConfig, UxQaReport, UxQaViewport } from "./types.js";

interface CliOptions {
  url: string;
  out: string | null;
  routeId: string | undefined;
  storyId: string | undefined;
  artifactsDir: string | undefined;
  screenshot: boolean;
  ariaSnapshot: boolean;
  viewports: UxQaViewport[];
  config: UxQaConfig;
}

const DEFAULT_VIEWPORTS: UxQaViewport[] = [
  { width: 390, height: 844 },
  { width: 1440, height: 900 }
];

export async function runCli(argv: string[]): Promise<number> {
  const options = parseArgs(argv);
  const browser = await chromium.launch();
  const reports: UxQaReport[] = [];
  for (const viewport of options.viewports) {
    const page = await browser.newPage({ viewport });
    await page.goto(options.url, { waitUntil: "networkidle" });
    const report = await analyzePage(page, options.config, {
      routeId: options.routeId,
      storyId: options.storyId,
      browserName: "chromium",
      artifactsDir: options.artifactsDir,
      screenshot: options.screenshot,
      ariaSnapshot: options.ariaSnapshot
    });
    await collectArtifacts(page, report, options);
    reports.push(report);
    await page.close();
  }
  await browser.close();
  await emitReport(reports, options.out);
  return reports.some((report) => report.violations.some((item) => item.severity === "error")) ? 1 : 0;
}

function parseArgs(argv: string[]): CliOptions {
  const args = argv[0] === "audit" ? argv.slice(1) : argv;
  const viewports: UxQaViewport[] = [];
  let url = "";
  let out: string | null = null;
  let routeId: string | undefined;
  let storyId: string | undefined;
  let artifactsDir: string | undefined;
  let screenshot = false;
  let ariaSnapshot = false;
  let decisionThreshold: UxQaConfig["decisionThreshold"];
  for (let index = 0; index < args.length; index += 1) {
    const value = args[index];
    if (value === "--url") url = requireValue(args, index += 1, "--url");
    else if (value === "--out") out = requireValue(args, index += 1, "--out");
    else if (value === "--route-id") routeId = requireValue(args, index += 1, "--route-id");
    else if (value === "--story-id") storyId = requireValue(args, index += 1, "--story-id");
    else if (value === "--artifacts-dir") artifactsDir = requireValue(args, index += 1, "--artifacts-dir");
    else if (value === "--screenshot") screenshot = true;
    else if (value === "--aria-snapshot") ariaSnapshot = true;
    else if (value === "--decision-threshold") decisionThreshold = parseDecisionThreshold(requireValue(args, index += 1, "--decision-threshold"));
    else if (value === "--viewport") viewports.push(parseViewport(requireValue(args, index += 1, "--viewport")));
    else throw new Error(`unknown argument: ${value ?? ""}`);
  }
  if (!url) throw new Error("missing required --url");
  const config: UxQaConfig = {};
  if (decisionThreshold) config.decisionThreshold = decisionThreshold;
  return {
    url,
    out,
    routeId,
    storyId,
    artifactsDir,
    screenshot,
    ariaSnapshot,
    viewports: viewports.length > 0 ? viewports : DEFAULT_VIEWPORTS,
    config
  };
}

function requireValue(args: string[], index: number, flag: string): string {
  const value = args[index];
  if (!value) throw new Error(`missing value for ${flag}`);
  return value;
}

function parseViewport(value: string): UxQaViewport {
  const match = /^(\d+)x(\d+)$/.exec(value);
  if (!match || !match[1] || !match[2]) throw new Error(`invalid viewport ${value}; expected WIDTHxHEIGHT`);
  return { width: Number.parseInt(match[1], 10), height: Number.parseInt(match[2], 10) };
}

function parseDecisionThreshold(value: string): UxQaConfig["decisionThreshold"] {
  if (value === "error" || value === "warning") return value;
  throw new Error(`invalid --decision-threshold ${value}; expected error or warning`);
}

async function collectArtifacts(page: Page, report: UxQaReport, options: CliOptions): Promise<void> {
  if (!options.artifactsDir && !options.screenshot && !options.ariaSnapshot) return;
  const directory = options.artifactsDir ?? "ux-qa-artifacts";
  await mkdir(directory, { recursive: true });
  const base = artifactBase(report);

  if (options.screenshot) {
    const path = join(directory, `${base}.png`);
    await page.screenshot({ path, fullPage: true });
    report.artifacts.push(artifact("screenshot", path, report));
  }

  if (options.ariaSnapshot) {
    const path = join(directory, `${base}.aria.yml`);
    const snapshot = await page.locator("body").ariaSnapshot();
    await writeFile(path, `${snapshot}\n`, "utf8");
    report.artifacts.push(artifact("aria-snapshot", path, report));
  }

  for (let index = 0; index < report.violations.length; index += 1) {
    const violation = report.violations[index];
    if (!violation?.box) continue;
    const path = join(directory, `${base}.${index + 1}.${violation.ruleId}.png`);
    const clip = {
      x: Math.max(0, Math.floor(violation.box.x)),
      y: Math.max(0, Math.floor(violation.box.y)),
      width: Math.max(1, Math.ceil(violation.box.width)),
      height: Math.max(1, Math.ceil(violation.box.height))
    };
    await page.screenshot({ path, clip });
    violation.artifactPath = path;
    report.artifacts.push({
      ...artifact("crop", path, report),
      selector: violation.selector,
      ruleId: violation.ruleId
    });
  }
}

function artifact(kind: UxQaArtifact["kind"], path: string, report: UxQaReport): UxQaArtifact {
  return { kind, path, viewport: report.viewport };
}

function artifactBase(report: UxQaReport): string {
  const identity = report.storyId ?? report.routeId ?? report.url;
  return safeFileName(`${identity}.${report.viewport.width}x${report.viewport.height}`);
}

function safeFileName(value: string): string {
  return value.replace(/[^a-z0-9._-]+/gi, "-").replace(/^-+|-+$/g, "").slice(0, 120) || "ux-qa";
}

async function emitReport(reports: UxQaReport[], out: string | null): Promise<void> {
  const payload = JSON.stringify({ reports }, null, 2);
  if (out) await writeFile(out, `${payload}\n`, "utf8");
  else process.stdout.write(`${payload}\n`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runCli(process.argv.slice(2)).then((code) => {
    process.exitCode = code;
  });
}
