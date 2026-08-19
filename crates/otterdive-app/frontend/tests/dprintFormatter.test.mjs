import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

import { createContext } from "@dprint/formatter";
import { getPath as getDockerfilePath } from "@dprint/dockerfile";
import { getPath as getJsonPath } from "@dprint/json";
import { getPath as getMarkdownPath } from "@dprint/markdown";
import { getBuffer as getRuffBuffer } from "@dprint/ruff";
import { getPath as getTomlPath } from "@dprint/toml";
import { getPath as getTypeScriptPath } from "@dprint/typescript";

const context = createContext({ indentWidth: 2, lineWidth: 100, newLineKind: "lf", useTabs: false });
for (const path of [
  getTypeScriptPath(),
  getJsonPath(),
  getMarkdownPath(),
  getTomlPath(),
  getDockerfilePath(),
]) {
  context.addPlugin(fs.readFileSync(path));
}
context.addPlugin(getRuffBuffer());

test("formats every language handled by the bundled dprint worker", () => {
  const examples = [
    ["component.tsx", "export const View=()=> <div>{1+2}</div>", /export const View = \(\) => <div>\{1 \+ 2\}<\/div>;/],
    ["config.json", "{\"name\":\"OtterDive\",\"enabled\":true}", /\{ "name": "OtterDive", "enabled": true \}/],
    ["config.jsonc", "{ // comment\n\"enabled\":true}", /\{ \/\/ comment\n  "enabled": true\n\}/],
    ["notes.md", "#  Title\n\n-   item", /# Title\n\n- item/],
    ["settings.toml", "name=\"OtterDive\"\nenabled=true", /name = "OtterDive"\nenabled = true/],
    ["script.py", "items=[1,2,3]\nprint(  items )", /items = \[1, 2, 3\]\nprint\(items\)/],
    ["Dockerfile", "FROM alpine:latest\nRUN  echo  hello", /FROM alpine:latest\nRUN echo hello/],
  ];

  for (const [filePath, fileText, expected] of examples) {
    const formatted = context.formatText({ filePath, fileText });
    assert.match(formatted, expected, `${filePath} should be formatted`);
  }
});
