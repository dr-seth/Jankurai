"""Rendered-UX QA readiness checks for the humanlint audit."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable


@dataclass(frozen=True)
class UxQaReadiness:
    web_surface: bool
    storybook: list[str]
    playwright_visual: list[str]
    visual_review: list[str]
    accessibility: list[str]
    layout_stability: list[str]
    api_mocks: list[str]
    design_tokens: list[str]
    geometry_runtime: list[str]

    @property
    def has_rendered_ux_lane(self) -> bool:
        if not self.web_surface:
            return True
        return bool(
            self.storybook
            and self.playwright_visual
            and self.accessibility
            and (self.visual_review or self.geometry_runtime)
        )

    @property
    def missing_categories(self) -> list[str]:
        if not self.web_surface:
            return []
        missing: list[str] = []
        for name, values in (
            ("Storybook state coverage", self.storybook),
            ("Playwright screenshot capture", self.playwright_visual),
            ("visual review or geometry runtime", self.visual_review or self.geometry_runtime),
            ("accessibility automation", self.accessibility),
            ("layout stability checks", self.layout_stability),
            ("generated API mocks", self.api_mocks),
            ("design token discipline", self.design_tokens),
        ):
            if not values:
                missing.append(name)
        return missing

    def as_dict(self) -> dict:
        return {
            "web_surface": self.web_surface,
            "has_rendered_ux_lane": self.has_rendered_ux_lane,
            "missing_categories": self.missing_categories,
            "evidence": {
                "storybook": self.storybook,
                "playwright_visual": self.playwright_visual,
                "visual_review": self.visual_review,
                "accessibility": self.accessibility,
                "layout_stability": self.layout_stability,
                "api_mocks": self.api_mocks,
                "design_tokens": self.design_tokens,
                "geometry_runtime": self.geometry_runtime,
            },
        }


def ux_qa_status(ctx, web_surface: bool) -> UxQaReadiness:
    files = list(getattr(ctx, "all_files", []))
    return UxQaReadiness(
        web_surface=web_surface,
        storybook=_paths_with(files, STORYBOOK_MARKERS, STORYBOOK_PATHS),
        playwright_visual=_paths_with(files, PLAYWRIGHT_VISUAL_MARKERS, ()),
        visual_review=_paths_with(files, VISUAL_REVIEW_MARKERS, VISUAL_REVIEW_PATHS),
        accessibility=_paths_with(files, ACCESSIBILITY_MARKERS, ()),
        layout_stability=_paths_with(files, LAYOUT_STABILITY_MARKERS, ()),
        api_mocks=_paths_with(files, API_MOCK_MARKERS, ()),
        design_tokens=_paths_with(files, DESIGN_TOKEN_MARKERS, DESIGN_TOKEN_PATHS),
        geometry_runtime=_paths_with(files, GEOMETRY_RUNTIME_MARKERS, GEOMETRY_RUNTIME_PATHS),
    )


def _paths_with(files: Iterable, markers: set[str], path_markers: tuple[str, ...], limit: int = 5) -> list[str]:
    matches: list[str] = []
    for file in files:
        rel_path = getattr(file, "rel_path", "")
        text = getattr(file, "lower", "") or ""
        path_hit = any(marker in rel_path.lower() for marker in path_markers)
        text_hit = any(marker in text for marker in markers)
        if path_hit or text_hit:
            matches.append(rel_path)
        if len(matches) >= limit:
            break
    return matches


STORYBOOK_PATHS = (".storybook/", ".stories.", ".story.")
STORYBOOK_MARKERS = {"@storybook", "storybook", "component story format", "csf"}

PLAYWRIGHT_VISUAL_MARKERS = {
    "tohavescreenshot",
    "tohave screenshot",
    "page.screenshot",
    "locator.screenshot",
    "visual comparisons",
    "screenshotpath",
}

VISUAL_REVIEW_PATHS = ("backstop", "loki", "argos", "chromatic", "percy", "applitools")
VISUAL_REVIEW_MARKERS = {
    "@argos-ci",
    "argos",
    "chromatic",
    "percy",
    "applitools",
    "backstopjs",
    "loki",
    "visual regression",
    "visual review",
}

ACCESSIBILITY_MARKERS = {
    "@axe-core",
    "axe-core",
    "pa11y",
    "storybook-addon-a11y",
    "eslint-plugin-jsx-a11y",
    "accessibility testing",
    "wcag",
}

LAYOUT_STABILITY_MARKERS = {
    "lighthouse",
    "lhci",
    "web-vitals",
    "cumulative layout shift",
    "layout shift",
    "cls",
}

API_MOCK_MARKERS = {
    "msw",
    "mock service worker",
    "msw-storybook-addon",
    "mockserviceworker",
    "orval",
}

DESIGN_TOKEN_PATHS = ("tokens/", "design-tokens", "style-dictionary")
DESIGN_TOKEN_MARKERS = {
    "design tokens",
    "design-token",
    "style dictionary",
    "style-dictionary",
    "figma variables",
    "semantic tokens",
}

GEOMETRY_RUNTIME_PATHS = ("packages/ux-qa", "ux-qa")
GEOMETRY_RUNTIME_MARKERS = {
    "@humanlint/ux-qa",
    "humanlint-ux-qa",
    "analyzepage",
    "expectnouxviolations",
    "edge clearance",
    "target size",
    "getboundingclientrect",
}
