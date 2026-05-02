import { readFile } from "node:fs/promises";
import type { UxQaConfig, UxQaRoute, UxQaViewport } from "./types.js";

export async function readUxQaConfig(path: string | undefined): Promise<UxQaConfig> {
  if (!path) return {};
  const text = await readFile(path, "utf8");
  if (path.endsWith(".json")) return JSON.parse(text) as UxQaConfig;
  return parseTomlSubset(text);
}

function parseTomlSubset(text: string): UxQaConfig {
  const config: UxQaConfig = {};
  const routes: UxQaRoute[] = [];
  const viewports: UxQaViewport[] = [];
  let currentRoute: Partial<UxQaRoute> | null = null;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    if (line === "[[routes]]") {
      if (currentRoute?.id && currentRoute.url) routes.push(currentRoute as UxQaRoute);
      currentRoute = {};
      continue;
    }
    const match = /^([A-Za-z0-9_-]+)\s*=\s*(.+)$/.exec(line);
    if (!match?.[1] || !match[2]) continue;
    const key = match[1];
    const value = parseValue(match[2]);
    if (currentRoute) {
      if (key === "viewport") {
        currentRoute.viewports = [...(currentRoute.viewports ?? []), parseViewport(String(value))];
      } else if (key === "viewports" && Array.isArray(value)) {
        currentRoute.viewports = value.map((item) => parseViewport(String(item)));
      } else {
        (currentRoute as Record<string, unknown>)[key] = value;
      }
    } else {
      if (key === "viewport") {
        viewports.push(parseViewport(String(value)));
      } else if (key === "viewports" && Array.isArray(value)) {
        viewports.push(...value.map((item) => parseViewport(String(item))));
      } else {
        (config as Record<string, unknown>)[key] = value;
      }
    }
  }
  if (currentRoute?.id && currentRoute.url) routes.push(currentRoute as UxQaRoute);
  if (routes.length > 0) config.routes = routes;
  if (viewports.length > 0) config.viewports = viewports;
  return config;
}

function parseValue(value: string): string | number | boolean | Array<string | number | boolean> {
  const trimmed = value.trim();
  if (trimmed === "true") return true;
  if (trimmed === "false") return false;
  if (/^\d+$/.test(trimmed)) return Number.parseInt(trimmed, 10);
  if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
    return JSON.parse(trimmed.replace(/,\s*]/g, "]")) as Array<string | number | boolean>;
  }
  return trimmed.replace(/^"|"$/g, "");
}

function parseViewport(value: string): UxQaViewport {
  const match = /^(\d+)x(\d+)$/.exec(value);
  if (!match?.[1] || !match[2]) throw new Error(`invalid viewport ${value}`);
  return { width: Number.parseInt(match[1], 10), height: Number.parseInt(match[2], 10) };
}
