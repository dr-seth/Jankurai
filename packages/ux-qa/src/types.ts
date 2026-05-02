export type UxQaSeverity = "error" | "warning";
export type UxQaDecision = "pass" | "warn" | "review" | "block";
export type UxQaState = "loading" | "empty" | "error" | "success" | "permission-denied";
export type UxQaBaselineMode = "pass" | "review" | "block";
export type UxQaReportSchemaVersion = "1.2.0" | "1.3.0";

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
  artifactRoot?: string;
  edgeClearancePx?: number;
  minimumTargetPx?: number;
  allowButtonWrap?: boolean;
  maximumZIndex?: number;
  allowNestedScrollbars?: boolean;
  decisionThreshold?: UxQaSeverity;
  readyState?: "domcontentloaded" | "load" | "networkidle";
  timeoutMs?: number;
  outputRoot?: string;
  storybookUrl?: string;
  requiredStates?: UxQaState[];
  visualBaselineMode?: UxQaBaselineMode;
  screenshotRequired?: boolean;
  ariaSnapshotRequired?: boolean;
  accessibilityScanRequired?: boolean;
  routes?: UxQaRoute[];
  viewports?: UxQaViewport[];
}

export interface UxQaRoute {
  id: string;
  url: string;
  storyId?: string;
  states?: UxQaState[];
  viewports?: UxQaViewport[];
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
  pointerEvents: string;
  disabled: boolean;
  inert: boolean;
  focusVisible: boolean;
  labelled: boolean;
  hitTargetSelector: string | null;
  obstructedBy: string | null;
  selectorResolved: boolean;
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

export type UxQaArtifactKind = "screenshot" | "crop" | "aria-snapshot" | "accessibility";

export interface UxQaArtifact {
  kind: UxQaArtifactKind;
  path: string;
  viewport: UxQaViewport;
  selector?: string;
  ruleId?: UxQaRuleId;
}

export interface UxQaArtifactCoverage {
  required: UxQaArtifactKind[];
  present: UxQaArtifactKind[];
  missing: UxQaArtifactKind[];
}

export interface UxQaAccessibilitySummary {
  violations: number;
  incomplete: number;
  passes: number;
  artifactPath?: string;
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
  accessibilityScan?: boolean | undefined;
  requiredStates?: UxQaState[] | undefined;
  declaredStates?: UxQaState[] | undefined;
}

export interface UxQaStateCoverage {
  required: UxQaState[];
  declared: UxQaState[];
  missing: UxQaState[];
}

export interface UxQaReport {
  schemaVersion: UxQaReportSchemaVersion;
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
  artifactCoverage?: UxQaArtifactCoverage;
  accessibility?: UxQaAccessibilitySummary;
  summary: UxQaSummary;
  stateCoverage?: UxQaStateCoverage;
  decision: UxQaDecision;
}
