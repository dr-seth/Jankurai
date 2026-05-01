#!/usr/bin/env node
import { writeFile } from "node:fs/promises";
import { chromium } from "playwright";
import { analyzePage } from "./page-analyzer.js";
import type { UxQaConfig, UxQaReport, UxQaViewport } from "./types.js";

interface CliOptions {
  url: string;
  out: string | null;
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
    reports.push(await analyzePage(page, options.config));
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
  for (let index = 0; index < args.length; index += 1) {
    const value = args[index];
    if (value === "--url") url = requireValue(args, index += 1, "--url");
    else if (value === "--out") out = requireValue(args, index += 1, "--out");
    else if (value === "--viewport") viewports.push(parseViewport(requireValue(args, index += 1, "--viewport")));
    else throw new Error(`unknown argument: ${value ?? ""}`);
  }
  if (!url) throw new Error("missing required --url");
  return { url, out, viewports: viewports.length > 0 ? viewports : DEFAULT_VIEWPORTS, config: {} };
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
