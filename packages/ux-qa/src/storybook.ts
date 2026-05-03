import type { UxQaRoute } from "./types.js";

export interface StorybookStory {
  id: string;
  title?: string;
  name?: string;
}

export async function discoverStorybookStories(baseUrl: string): Promise<StorybookStory[]> {
  const root = baseUrl.replace(/\/+$/, "");
  const response = await fetch(`${root}/index.json`);
  if (!response.ok) throw new Error(`failed to read Storybook index: ${response.status}`);
  const payload = (await response.json()) as { entries?: Record<string, StorybookStory>; stories?: Record<string, StorybookStory> };
  const entries = payload.entries ?? payload.stories ?? {};
  return Object.entries(entries).map(([id, story]) => ({ ...story, id: story.id ?? id }));
}

export function storybookIframeUrl(baseUrl: string, storyId: string): string {
  return `${baseUrl.replace(/\/+$/, "")}/iframe.html?id=${encodeURIComponent(storyId)}`;
}

export function resolveStorybookRoutes(
  baseUrl: string,
  stories: StorybookStory[],
  configuredRoutes?: UxQaRoute[]
): UxQaRoute[] {
  if (!configuredRoutes?.length) {
    return stories.map((story) => ({
      id: story.id,
      storyId: story.id,
      url: storybookIframeUrl(baseUrl, story.id)
    }));
  }

  const missingStoryId = configuredRoutes.find((route) => !route.storyId);
  if (missingStoryId) {
    throw new Error(`configured Storybook route ${missingStoryId.id} is missing storyId`);
  }

  const storiesById = new Map(stories.map((story) => [story.id, story]));
  return configuredRoutes.map((route) => {
    const story = storiesById.get(route.storyId!);
    if (!story) {
      throw new Error(`configured Storybook route ${route.id} references missing storyId ${route.storyId}`);
    }
    return {
      ...route,
      storyId: story.id,
      url: storybookIframeUrl(baseUrl, story.id)
    };
  });
}
