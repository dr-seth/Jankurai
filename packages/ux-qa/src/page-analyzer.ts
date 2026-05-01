import type { Page } from "playwright";
import { collectPageMetrics, collectUxElements, collectViewport } from "./collector.js";
import { UxQaAssertionError } from "./errors.js";
import { runUxRules } from "./rules.js";
import type { UxQaConfig, UxQaReport } from "./types.js";

export async function analyzePage(page: Page, config: UxQaConfig = {}): Promise<UxQaReport> {
  const [viewport, metrics, elements] = await Promise.all([
    collectViewport(page),
    collectPageMetrics(page),
    collectUxElements(page)
  ]);
  return {
    url: page.url(),
    checkedAt: new Date().toISOString(),
    viewport,
    metrics,
    elements,
    violations: runUxRules(elements, viewport, metrics, config)
  };
}

export async function expectNoUxViolations(page: Page, config: UxQaConfig = {}): Promise<void> {
  const report = await analyzePage(page, config);
  if (report.violations.length === 0) return;
  const summary = report.violations.map((item) => `${item.ruleId} ${item.selector}: ${item.evidence}`).join("\n");
  throw new UxQaAssertionError(`humanlint UX QA found ${report.violations.length} violation(s)\n${summary}`);
}
