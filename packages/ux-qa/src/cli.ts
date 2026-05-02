#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { chromium } from "playwright";
import type { Page } from "playwright";
import { runAccessibilityScan, summarizeAccessibility } from "./accessibility.js";
import { readUxQaConfig } from "./config.js";
import { analyzePage } from "./page-analyzer.js";
import { reportArtifactPath } from "./receipts.js";
import { discoverStorybookStories, storybookIframeUrl } from "./storybook.js";
import type {
  UxQaArtifact,
  UxQaArtifactCoverage,
  UxQaArtifactKind,
  UxQaConfig,
  UxQaDecision,
  UxQaReport,
  UxQaRoute,
  UxQaViewport
} from "./types.js";

interface CliOptions {
  command: "audit" | "storybook";
  url: string;
  out: string | null;
  routeId: string | undefined;
  storyId: string | undefined;
  artifactsDir: string | undefined;
  screenshot: boolean;
  ariaSnapshot: boolean;
  accessibilityScan: boolean;
  viewports: UxQaViewport[];
  configPath: string | undefined;
  config: UxQaConfig;
  waitFor: "domcontentloaded" | "load" | "networkidle";
  timeoutMs: number;
}

const DEFAULT_VIEWPORTS: UxQaViewport[] = [
  { width: 390, height: 844 },
  { width: 1440, height: 900 }
];

export async function runCli(argv: string[]): Promise<number> {
  const options = await parseArgs(argv);
  const browser = await chromium.launch();
  const reports: UxQaReport[] = [];
  try {
    const routes = await routesForOptions(options);
    for (const route of routes) {
      const viewports = route.viewports?.length ? route.viewports : options.viewports;
      for (const viewport of viewports) {
        const page = await browser.newPage({ viewport });
        try {
          await page.goto(route.url, { waitUntil: options.waitFor, timeout: options.timeoutMs });
          const report = await analyzePage(page, options.config, {
            routeId: route.id,
            storyId: route.storyId,
            browserName: "chromium",
            artifactsDir: options.artifactsDir,
            screenshot: options.screenshot,
            ariaSnapshot: options.ariaSnapshot,
            requiredStates: options.config.requiredStates,
            declaredStates: route.states
          });
          await collectArtifacts(page, report, options);
          applyEvidenceDecision(report);
          reports.push(report);
        } finally {
          await page.close();
        }
      }
    }
  } finally {
    await browser.close();
  }
  await emitReport(reports, options.out);
  return reports.some((report) => report.decision === "block") ? 1 : 0;
}

async function parseArgs(argv: string[]): Promise<CliOptions> {
  const command = argv[0] === "storybook" ? "storybook" : "audit";
  const args = argv[0] === "audit" || argv[0] === "storybook" ? argv.slice(1) : argv;
  const viewports: UxQaViewport[] = [];
  let url = "";
  let out: string | null = null;
  let routeId: string | undefined;
  let storyId: string | undefined;
  let artifactsDir: string | undefined;
  let screenshot = false;
  let ariaSnapshot = false;
  let accessibilityScan = false;
  let decisionThreshold: UxQaConfig["decisionThreshold"];
  let waitForOverride: CliOptions["waitFor"] | undefined;
  let timeoutMsOverride: number | undefined;
  let configPath: string | undefined;
  for (let index = 0; index < args.length; index += 1) {
    const value = args[index];
    if (value === "--url") url = requireValue(args, index += 1, "--url");
    else if (value === "--out") out = requireValue(args, index += 1, "--out");
    else if (value === "--config") configPath = requireValue(args, index += 1, "--config");
    else if (value === "--route-id") routeId = requireValue(args, index += 1, "--route-id");
    else if (value === "--story-id") storyId = requireValue(args, index += 1, "--story-id");
    else if (value === "--artifacts-dir") artifactsDir = requireValue(args, index += 1, "--artifacts-dir");
    else if (value === "--screenshot") screenshot = true;
    else if (value === "--aria-snapshot") ariaSnapshot = true;
    else if (value === "--accessibility-scan") accessibilityScan = true;
    else if (value === "--decision-threshold") decisionThreshold = parseDecisionThreshold(requireValue(args, index += 1, "--decision-threshold"));
    else if (value === "--wait-for") waitForOverride = parseWaitFor(requireValue(args, index += 1, "--wait-for"));
    else if (value === "--timeout-ms") timeoutMsOverride = parseTimeoutMs(requireValue(args, index += 1, "--timeout-ms"));
    else if (value === "--viewport") viewports.push(parseViewport(requireValue(args, index += 1, "--viewport")));
    else throw new Error(`unknown argument: ${value ?? ""}`);
  }
  const fileConfig = await readUxQaConfig(configPath);
  const config: UxQaConfig = { ...fileConfig };
  if (decisionThreshold) config.decisionThreshold = decisionThreshold;
  const waitFor = waitForOverride ?? fileConfig.readyState ?? "domcontentloaded";
  const timeoutMs = timeoutMsOverride ?? fileConfig.timeoutMs ?? 15000;
  config.readyState = waitFor;
  config.timeoutMs = timeoutMs;
  if (!artifactsDir && fileConfig.artifactRoot) artifactsDir = fileConfig.artifactRoot;
  if (!url && command === "storybook" && fileConfig.storybookUrl) url = fileConfig.storybookUrl;
  screenshot = screenshot || fileConfig.screenshotRequired === true;
  ariaSnapshot = ariaSnapshot || fileConfig.ariaSnapshotRequired === true;
  accessibilityScan = accessibilityScan || fileConfig.accessibilityScanRequired === true;
  if (!url && !config.routes?.length && command !== "storybook") throw new Error("missing required --url or config routes");
  if (!url && command === "storybook") throw new Error("missing required --url or storybookUrl for Storybook");
  return {
    command,
    url,
    out,
    routeId,
    storyId,
    artifactsDir,
    screenshot,
    ariaSnapshot,
    accessibilityScan,
    viewports: viewports.length > 0 ? viewports : config.viewports ?? DEFAULT_VIEWPORTS,
    config,
    configPath,
    waitFor,
    timeoutMs
  };
}

async function routesForOptions(options: CliOptions): Promise<UxQaRoute[]> {
  if (options.command === "storybook") {
    const stories = await discoverStorybookStories(options.url);
    return stories.map((story) => ({
      id: story.id,
      storyId: story.id,
      url: storybookIframeUrl(options.url, story.id)
    }));
  }
  if (options.config.routes?.length) return options.config.routes;
  return [{
    id: options.routeId ?? options.url,
    url: options.url,
    ...(options.storyId ? { storyId: options.storyId } : {})
  }];
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

function parseWaitFor(value: string): CliOptions["waitFor"] {
  if (value === "domcontentloaded" || value === "load" || value === "networkidle") return value;
  throw new Error(`invalid --wait-for ${value}; expected domcontentloaded, load, or networkidle`);
}

function parseTimeoutMs(value: string): number {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) throw new Error(`invalid --timeout-ms ${value}; expected a positive integer`);
  return parsed;
}

async function collectArtifacts(page: Page, report: UxQaReport, options: CliOptions): Promise<void> {
  const requiredKinds = requiredArtifactKinds(options);
  if (!options.artifactsDir && !options.screenshot && !options.ariaSnapshot && !options.accessibilityScan) {
    report.artifactCoverage = artifactCoverage(requiredKinds, report.artifacts);
    return;
  }
  const directory = options.artifactsDir ?? "ux-qa-artifacts";
  await mkdir(directory, { recursive: true });
  const base = artifactBase(report);
  const outputRoot = options.config.outputRoot ?? process.cwd();

  if (options.screenshot) {
    const path = join(directory, `${base}.png`);
    await page.screenshot({ path, fullPage: true });
    report.artifacts.push(artifact("screenshot", path, report, outputRoot));
  }

  if (options.ariaSnapshot) {
    const path = join(directory, `${base}.aria.yml`);
    const snapshot = await page.locator("body").ariaSnapshot();
    await writeFile(path, `${snapshot}\n`, "utf8");
    report.artifacts.push(artifact("aria-snapshot", path, report, outputRoot));
  }

  if (options.accessibilityScan) {
    const path = join(directory, `${base}.a11y.json`);
    const result = await runAccessibilityScan(page);
    const artifactPath = reportArtifactPath(path, outputRoot);
    await writeFile(path, `${JSON.stringify(result, null, 2)}\n`, "utf8");
    report.accessibility = summarizeAccessibility(result, artifactPath);
    report.artifacts.push(artifact("accessibility", path, report, outputRoot));
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
    violation.artifactPath = reportArtifactPath(path, outputRoot);
    report.artifacts.push({
      ...artifact("crop", path, report, outputRoot),
      selector: violation.selector,
      ruleId: violation.ruleId
    });
  }

  report.artifactCoverage = artifactCoverage(requiredKinds, report.artifacts);
}

function artifact(kind: UxQaArtifact["kind"], path: string, report: UxQaReport, outputRoot: string): UxQaArtifact {
  return { kind, path: reportArtifactPath(path, outputRoot), viewport: report.viewport };
}

function requiredArtifactKinds(options: CliOptions): UxQaArtifactKind[] {
  const required: UxQaArtifactKind[] = [];
  if (options.config.screenshotRequired) required.push("screenshot");
  if (options.config.ariaSnapshotRequired) required.push("aria-snapshot");
  if (options.config.accessibilityScanRequired) required.push("accessibility");
  return required;
}

function artifactCoverage(required: UxQaArtifactKind[], artifacts: UxQaArtifact[]): UxQaArtifactCoverage {
  const present = uniqueKinds(artifacts.map((item) => item.kind));
  return {
    required,
    present,
    missing: required.filter((kind) => !present.includes(kind))
  };
}

function uniqueKinds(kinds: UxQaArtifactKind[]): UxQaArtifactKind[] {
  const out: UxQaArtifactKind[] = [];
  for (const kind of kinds) {
    if (!out.includes(kind)) out.push(kind);
  }
  return out;
}

function applyEvidenceDecision(report: UxQaReport): void {
  let decision = report.decision;
  if (report.stateCoverage?.missing.length) decision = mergeDecision(decision, "block");
  if (report.artifactCoverage?.missing.length) decision = mergeDecision(decision, "block");
  if ((report.accessibility?.violations ?? 0) > 0) decision = mergeDecision(decision, "block");
  if ((report.accessibility?.incomplete ?? 0) > 0) decision = mergeDecision(decision, "review");
  report.decision = decision;
}

function mergeDecision(current: UxQaDecision, next: UxQaDecision): UxQaDecision {
  return decisionRank(next) > decisionRank(current) ? next : current;
}

function decisionRank(decision: UxQaDecision): number {
  switch (decision) {
    case "block":
      return 4;
    case "review":
      return 3;
    case "warn":
      return 2;
    case "pass":
      return 1;
  }
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
  if (out) {
    await mkdir(dirname(out), { recursive: true });
    await writeFile(out, `${payload}\n`, "utf8");
  } else {
    process.stdout.write(`${payload}\n`);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runCli(process.argv.slice(2)).then((code) => {
    process.exitCode = code;
  });
}
