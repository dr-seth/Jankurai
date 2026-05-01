export type UxQaSeverity = "error" | "warning";
export type UxQaDecision = "pass" | "warn" | "review" | "block";

export type UxQaRuleId =
  | "edge-clearance"
  | "target-size"
  | "interactive-overlap"
  | "text-clipping"
  | "button-wrap"
  | "horizontal-overflow"
  | "sticky-obstruction"
  | "z-index-token"
  | "focus-visible"
  | "form-label"
  | "nested-scrollbar";

export interface UxQaViewport {
  width: number;
  height: number;
}

export interface UxQaBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface UxQaConfig {
  edgeClearancePx?: number;
  minimumTargetPx?: number;
  allowButtonWrap?: boolean;
  maximumZIndex?: number;
  allowNestedScrollbars?: boolean;
  decisionThreshold?: UxQaSeverity;
}

export interface UxQaElement {
  selector: string;
  tag: string;
  role: string | null;
  interactive: boolean;
  name: string;
  text: string;
  box: UxQaBox;
  lineCount: number;
  scrollWidth: number;
  scrollHeight: number;
  clientWidth: number;
  clientHeight: number;
  overflowX: string;
  overflowY: string;
  position: string;
  zIndex: string;
  focusVisible: boolean;
  labelled: boolean;
}

export interface UxQaPageMetrics {
  scrollWidth: number;
  clientWidth: number;
  scrollHeight: number;
  clientHeight: number;
}

export interface UxQaViolation {
  ruleId: UxQaRuleId;
  severity: UxQaSeverity;
  message: string;
  selector: string;
  evidence: string;
  box?: UxQaBox;
  artifactPath?: string;
}

export interface UxQaArtifact {
  kind: "screenshot" | "crop" | "aria-snapshot";
  path: string;
  viewport: UxQaViewport;
  selector?: string;
  ruleId?: UxQaRuleId;
}

export interface UxQaSummary {
  errors: number;
  warnings: number;
  byRule: Partial<Record<UxQaRuleId, number>>;
}

export interface UxQaRunContext {
  routeId?: string | undefined;
  storyId?: string | undefined;
  browserName?: string | undefined;
  artifactsDir?: string | undefined;
  screenshot?: boolean | undefined;
  ariaSnapshot?: boolean | undefined;
}

export interface UxQaReport {
  schemaVersion: "1.0.0";
  toolVersion: string;
  url: string;
  routeId?: string;
  storyId?: string;
  browserName?: string;
  checkedAt: string;
  viewport: UxQaViewport;
  metrics: UxQaPageMetrics;
  elements: UxQaElement[];
  violations: UxQaViolation[];
  artifacts: UxQaArtifact[];
  summary: UxQaSummary;
  decision: UxQaDecision;
}
