#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { chromium } from "playwright";
import type { Page } from "playwright";
import { readUxQaConfig } from "./config.js";
import { analyzePage } from "./page-analyzer.js";
import { reportArtifactPath } from "./receipts.js";
import { discoverStorybookStories, storybookIframeUrl } from "./storybook.js";
import type { UxQaArtifact, UxQaConfig, UxQaReport, UxQaRoute, UxQaViewport } from "./types.js";

interface CliOptions {
  command: "audit" | "storybook";
  url: string;
  out: string | null;
  routeId: string | undefined;
  storyId: string | undefined;
  artifactsDir: string | undefined;
  screenshot: boolean;
  ariaSnapshot: boolean;
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
  return reports.some((report) => report.violations.some((item) => item.severity === "error")) ? 1 : 0;
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
  let decisionThreshold: UxQaConfig["decisionThreshold"];
  let waitFor: CliOptions["waitFor"] = "domcontentloaded";
  let timeoutMs = 15000;
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
    else if (value === "--decision-threshold") decisionThreshold = parseDecisionThreshold(requireValue(args, index += 1, "--decision-threshold"));
    else if (value === "--wait-for") waitFor = parseWaitFor(requireValue(args, index += 1, "--wait-for"));
    else if (value === "--timeout-ms") timeoutMs = parseTimeoutMs(requireValue(args, index += 1, "--timeout-ms"));
    else if (value === "--viewport") viewports.push(parseViewport(requireValue(args, index += 1, "--viewport")));
    else throw new Error(`unknown argument: ${value ?? ""}`);
  }
  const fileConfig = await readUxQaConfig(configPath);
  const config: UxQaConfig = { ...fileConfig };
  if (decisionThreshold) config.decisionThreshold = decisionThreshold;
  config.readyState = waitFor;
  config.timeoutMs = timeoutMs;
  if (!artifactsDir && fileConfig.artifactRoot) artifactsDir = fileConfig.artifactRoot;
  if (!url && !config.routes?.length && command !== "storybook") throw new Error("missing required --url or config routes");
  if (!url && command === "storybook") throw new Error("missing required --url for Storybook");
  return {
    command,
    url,
    out,
    routeId,
    storyId,
    artifactsDir,
    screenshot,
    ariaSnapshot,
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
  if (!options.artifactsDir && !options.screenshot && !options.ariaSnapshot) return;
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
}

function artifact(kind: UxQaArtifact["kind"], path: string, report: UxQaReport, outputRoot: string): UxQaArtifact {
  return { kind, path: reportArtifactPath(path, outputRoot), viewport: report.viewport };
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
