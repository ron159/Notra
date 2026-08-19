import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import ts from "typescript";

const languageSupport = await loadTypeScriptModule("../src/languageSupport.ts");
const formatterSupport = await loadTypeScriptModule("../src/formatterSupport.ts");

const registry = [
  { id: "custom", extensions: [".custom"], filenames: ["CUSTOMFILE"] },
];

test("resolves common source and configuration file extensions", () => {
  assert.equal(languageSupport.languageFromFilePath("src/main.tsx", registry), "typescript");
  assert.equal(languageSupport.languageFromFilePath("C:\\work\\tool.py", registry), "python");
  assert.equal(languageSupport.languageFromFilePath("compose.yml", registry), "yaml");
  assert.equal(languageSupport.languageFromFilePath("schema.graphql", registry), "graphql");
  assert.equal(languageSupport.languageFromFilePath("header.hpp", registry), "cpp");
});

test("resolves extensionless and Monaco-registered file names", () => {
  assert.equal(languageSupport.languageFromFilePath("services/api/Dockerfile", registry), "dockerfile");
  assert.equal(languageSupport.languageFromFilePath("Makefile", registry), "shell");
  assert.equal(languageSupport.languageFromFilePath("CUSTOMFILE", registry), "custom");
  assert.equal(languageSupport.languageFromFilePath("sample.custom", registry), "custom");
  assert.equal(languageSupport.languageFromFilePath("unknown.extension", registry), "plaintext");
});

test("routes supported languages to a matching formatter file name", () => {
  assert.equal(formatterSupport.supportsDprintLanguage("typescript"), true);
  assert.equal(formatterSupport.supportsDprintLanguage("rust"), false);
  assert.equal(formatterSupport.formatterFilePath("typescript", "/tmp/component.tsx"), "component.tsx");
  assert.equal(formatterSupport.formatterFilePath("typescript", null), "untitled.ts");
  assert.equal(formatterSupport.formatterFilePath("dockerfile", "/tmp/Containerfile"), "Dockerfile");
});

test("preserves whether the source ended with a newline", () => {
  assert.equal(formatterSupport.preserveTrailingNewline("const value = 1;\n", "const value=1"), "const value = 1;");
  assert.equal(formatterSupport.preserveTrailingNewline("const value = 1;", "const value=1\r\n"), "const value = 1;\r\n");
  assert.equal(formatterSupport.preserveTrailingNewline("key = 1\n\n", "key=1\n"), "key = 1\n");
});

async function loadTypeScriptModule(relativePath) {
  const source = fs.readFileSync(new URL(relativePath, import.meta.url), "utf8");
  const javascript = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
    },
  }).outputText;
  return import(`data:text/javascript;base64,${Buffer.from(javascript).toString("base64")}`);
}
