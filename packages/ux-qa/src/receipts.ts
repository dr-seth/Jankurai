import { isAbsolute, relative } from "node:path";

export function reportArtifactPath(path: string, root = process.cwd()): string {
  if (!isAbsolute(path)) return path.replace(/\\/g, "/");
  const rel = relative(root, path).replace(/\\/g, "/");
  if (rel.startsWith("..")) return path.replace(/\\/g, "/");
  return rel;
}
