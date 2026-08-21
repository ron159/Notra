import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import ts from "typescript";

const { mapWithConcurrency } = await loadTypeScriptModule("../src/openBatch.ts");

test("keeps output order while limiting concurrent work", async () => {
  let active = 0;
  let peak = 0;
  const results = await mapWithConcurrency(
    [30, 5, 15, 1],
    async (delay, index) => {
      active += 1;
      peak = Math.max(peak, active);
      await new Promise((resolve) => setTimeout(resolve, delay));
      active -= 1;
      return `item-${index}`;
    },
    { concurrency: 2 },
  );

  assert.equal(peak, 2);
  assert.deepEqual(results.map((result) => result.value), ["item-0", "item-1", "item-2", "item-3"]);
});

test("starts the priority item first without changing output order", async () => {
  const started = [];
  const results = await mapWithConcurrency(
    ["first", "active", "last"],
    async (value) => {
      started.push(value);
      return value.toUpperCase();
    },
    { concurrency: 1, priorityIndex: 1 },
  );

  assert.deepEqual(started, ["active", "first", "last"]);
  assert.deepEqual(results.map((result) => result.value), ["FIRST", "ACTIVE", "LAST"]);
});

test("isolates failures so the rest of the batch completes", async () => {
  const progress = [];
  const results = await mapWithConcurrency(
    [1, 2, 3],
    async (value) => {
      if (value === 2) throw new Error("broken");
      return value * 2;
    },
    { concurrency: 3, onProgress: ({ completed }) => progress.push(completed) },
  );

  assert.equal(results[0].value, 2);
  assert.match(String(results[1].error), /broken/);
  assert.equal(results[2].value, 6);
  assert.deepEqual(progress, [1, 2, 3]);
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
