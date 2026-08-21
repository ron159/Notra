import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import ts from "typescript";

const performanceModule = await loadTypeScriptModule("../src/openPerformance.ts");
const mainSource = fs.readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");

test("normalizes collected batch metrics", () => {
  assert.deepEqual(
    performanceModule.summarizeOpenPerformance({
      durationMs: 83.4,
      failed: -1,
      heapDeltaBytes: 1024,
      longTaskCount: 2.8,
      longTaskDurationMs: 57.2,
      opened: 9.8,
      total: 10.9,
    }),
    {
      durationMs: 83.4,
      failed: 0,
      heapDeltaBytes: 1024,
      longTaskCount: 2,
      longTaskDurationMs: 57.2,
      opened: 9,
      total: 10,
    },
  );
});

test("multi-file entry points use one deferred batch instead of per-file activation", () => {
  assert.match(mainSource, /openDroppedFiles[\s\S]*?await openPaths\(paths,/);
  assert.match(mainSource, /applyOpenRequest[\s\S]*?await openPaths\(args\.files,/);
  assert.match(mainSource, /addOrReplaceDocument\(result\.value, origin, \{ activate: false, deferModel: true \}\)/);
  assert.doesNotMatch(mainSource, /for \(const path of uniquePaths\)[\s\S]{0,180}await openPath\(path, true\)/);
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
