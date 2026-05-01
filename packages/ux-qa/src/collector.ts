import type { Page } from "playwright";
import type { UxQaElement, UxQaPageMetrics, UxQaViewport } from "./types.js";

const INTERACTIVE_SELECTOR = [
  "button",
  "a[href]",
  "input",
  "select",
  "textarea",
  "summary",
  "[role='button']",
  "[role='link']",
  "[role='menuitem']",
  "[role='tab']",
  "[role='checkbox']",
  "[role='radio']",
  "[tabindex]:not([tabindex='-1'])"
].join(",");

const LAYOUT_SELECTOR = [
  "header",
  "footer",
  "main",
  "nav",
  "section",
  "article",
  "form",
  "[data-ux-qa-region]",
  "[style*='overflow']",
  "[class*='overflow']",
  "[style*='position: fixed']",
  "[style*='position:fixed']",
  "[style*='position: sticky']",
  "[style*='position:sticky']"
].join(",");

export async function collectViewport(page: Page): Promise<UxQaViewport> {
  const viewport = page.viewportSize();
  if (viewport) return viewport;
  return page.evaluate(() => ({ width: window.innerWidth, height: window.innerHeight }));
}

export async function collectPageMetrics(page: Page): Promise<UxQaPageMetrics> {
  return page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
    scrollHeight: document.documentElement.scrollHeight,
    clientHeight: document.documentElement.clientHeight
  }));
}

type CollectArgs = {
  selector: string;
  interactiveSelector: string;
};

type ElementState = Pick<UxQaElement, "focusVisible" | "labelled">;

function collectElementBoxesInBrowser({ selector, interactiveSelector }: CollectArgs): UxQaElement[] {
  function isVisible(node: HTMLElement): boolean {
    const style = window.getComputedStyle(node);
    const rect = node.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0 && style.display !== "none" && style.visibility !== "hidden";
  }

  function cssEscape(value: string): string {
    return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  }

  function stableSelector(node: HTMLElement, index: number): string {
    const tag = node.tagName.toLowerCase();
    const testId = node.getAttribute("data-testid");
    const label = node.getAttribute("aria-label");
    if (testId) return `[data-testid="${cssEscape(testId)}"]`;
    if (node.id) return `#${cssEscape(node.id)}`;
    if (label) return `${tag}[aria-label="${cssEscape(label)}"]`;
    return `${tag}:nth-of-type(${index + 1})`;
  }

  function countTextLines(node: HTMLElement): number {
    const range = document.createRange();
    range.selectNodeContents(node);
    const lineCount = Array.from(range.getClientRects()).filter((rect) => rect.width > 0 && rect.height > 0).length;
    range.detach();
    return Math.max(1, lineCount);
  }

  function describeElement(node: HTMLElement, index: number, interactive: boolean): UxQaElement {
    const rect = node.getBoundingClientRect();
    const style = window.getComputedStyle(node);
    const text = (node.textContent ?? "").replace(/\s+/g, " ").trim().slice(0, 120);
    return {
      selector: stableSelector(node, index),
      tag: node.tagName.toLowerCase(),
      role: node.getAttribute("role"),
      interactive,
      name: node.getAttribute("aria-label") ?? node.getAttribute("title") ?? "",
      text,
      box: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
      lineCount: countTextLines(node),
      scrollWidth: node.scrollWidth,
      scrollHeight: node.scrollHeight,
      clientWidth: node.clientWidth,
      clientHeight: node.clientHeight,
      overflowX: style.overflowX,
      overflowY: style.overflowY,
      position: style.position,
      zIndex: style.zIndex,
      focusVisible: true,
      labelled: true
    };
  }

  const nodes = Array.from(document.querySelectorAll<HTMLElement>(selector));
  return nodes.filter(isVisible).map((node, index) => describeElement(node, index, node.matches(interactiveSelector)));
}

function collectElementStatesInBrowser({ selector }: CollectArgs): ElementState[] {
  function isVisible(node: HTMLElement): boolean {
    const style = window.getComputedStyle(node);
    const rect = node.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0 && style.display !== "none" && style.visibility !== "hidden";
  }

  function hasVisibleFocus(node: HTMLElement): boolean {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    node.focus({ preventScroll: true });
    if (document.activeElement !== node) return true;

    const focusedStyle = window.getComputedStyle(node);
    const outlineWidth = Number.parseFloat(focusedStyle.outlineWidth || "0");
    const hasOutline = focusedStyle.outlineStyle !== "none" && outlineWidth > 0;
    const hasShadow = focusedStyle.boxShadow !== "none";
    const hasVisibleIndicator = hasOutline || hasShadow;

    previous?.focus({ preventScroll: true });
    if (!previous) node.blur();
    return hasVisibleIndicator;
  }

  function hasFormLabel(node: HTMLElement): boolean {
    const isField = node instanceof HTMLInputElement || node instanceof HTMLSelectElement || node instanceof HTMLTextAreaElement;
    if (!isField || (node instanceof HTMLInputElement && node.type === "hidden")) return true;
    if (node.getAttribute("aria-label") || node.getAttribute("aria-labelledby") || node.getAttribute("title")) return true;
    return (node.labels?.length ?? 0) > 0;
  }

  return Array.from(document.querySelectorAll<HTMLElement>(selector))
    .filter(isVisible)
    .map((node) => ({
      focusVisible: hasVisibleFocus(node),
      labelled: hasFormLabel(node)
    }));
}

export async function collectUxElements(page: Page): Promise<UxQaElement[]> {
  const args = { selector: `${INTERACTIVE_SELECTOR},${LAYOUT_SELECTOR}`, interactiveSelector: INTERACTIVE_SELECTOR };
  const elements = await page.evaluate(collectElementBoxesInBrowser, args);
  const states = await page.evaluate(collectElementStatesInBrowser, args);
  return elements.map((element, index) => ({ ...element, ...states[index] }));
}
