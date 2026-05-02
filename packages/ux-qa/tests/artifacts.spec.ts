import { expect, test } from "@playwright/test";
import { readFile, writeFile } from "node:fs/promises";
import { isAbsolute } from "node:path";
import { pathToFileURL } from "node:url";
import { runCli } from "../src/cli.js";

test("artifact paths are report-relative", async ({}, testInfo) => {
  const pagePath = testInfo.outputPath("fixture.html");
  const reportPath = testInfo.outputPath("ux-qa.json");
  const artifactsDir = testInfo.outputPath("artifacts");
  await writeFile(pagePath, `
    <button data-testid="tiny" style="position:absolute; left:2px; top:2px; width:12px; height:12px">x</button>
    <button data-testid="neighbor" style="position:absolute; left:18px; top:2px; width:12px; height:12px">y</button>
  `, "utf8");

  await runCli([
    "audit",
    "--url",
    pathToFileURL(pagePath).toString(),
    "--out",
    reportPath,
    "--artifacts-dir",
    artifactsDir,
    "--screenshot"
  ]);

  const payload = JSON.parse(await readFile(reportPath, "utf8"));
  const paths = payload.reports.flatMap((report: { artifacts: { path: string }[] }) => report.artifacts.map((item) => item.path));
  expect(paths.length).toBeGreaterThan(0);
  expect(paths.every((item: string) => !isAbsolute(item))).toBe(true);
});
