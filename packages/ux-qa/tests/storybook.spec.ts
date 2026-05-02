import { expect, test } from "@playwright/test";
import { discoverStorybookStories, storybookIframeUrl } from "../src/index.js";

test("Storybook index discovery supports index.json", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => new Response(JSON.stringify({
    entries: {
      "button--primary": { id: "button--primary", title: "Button", name: "Primary" }
    }
  }), { status: 200 })) as typeof fetch;
  try {
    const stories = await discoverStorybookStories("http://localhost:6006/");
    expect(stories).toEqual([{ id: "button--primary", title: "Button", name: "Primary" }]);
    expect(storybookIframeUrl("http://localhost:6006/", "button--primary")).toBe("http://localhost:6006/iframe.html?id=button--primary");
  } finally {
    globalThis.fetch = originalFetch;
  }
});
