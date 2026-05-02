import { expect, test } from "@playwright/test";
import { writeFile } from "node:fs/promises";
import { readUxQaConfig } from "../src/index.js";

test("config parses route matrix and viewports", async ({}, testInfo) => {
  const path = testInfo.outputPath("ux-qa.toml");
  await writeFile(path, `
artifactRoot = "target/humanlint/ux-qa"
storybookUrl = "http://localhost:6006"
requiredStates = ["loading", "success"]
viewports = ["390x844", "1440x900"]

[[routes]]
id = "dashboard"
url = "http://localhost:3000/dashboard"
states = ["loading", "success"]
viewports = ["390x844"]
`, "utf8");

  const config = await readUxQaConfig(path);

  expect(config.artifactRoot).toBe("target/humanlint/ux-qa");
  expect(config.storybookUrl).toBe("http://localhost:6006");
  expect(config.requiredStates).toEqual(["loading", "success"]);
  expect(config.viewports).toEqual([{ width: 390, height: 844 }, { width: 1440, height: 900 }]);
  expect(config.routes?.[0]).toEqual({
    id: "dashboard",
    url: "http://localhost:3000/dashboard",
    states: ["loading", "success"],
    viewports: [{ width: 390, height: 844 }]
  });
});
