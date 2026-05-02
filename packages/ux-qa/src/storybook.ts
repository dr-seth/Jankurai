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
