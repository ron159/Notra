import assert from "node:assert/strict";

import { resolveAnalyseEnterAction } from "../src/analyseState.ts";

assert.equal(resolveAnalyseEnterAction(false, true, "search"), "add");
assert.equal(resolveAnalyseEnterAction(false, false, "update"), "add");
assert.equal(resolveAnalyseEnterAction(true, false, "add"), "search");
assert.equal(resolveAnalyseEnterAction(true, true, "search"), "search");
assert.equal(resolveAnalyseEnterAction(true, true, "update"), "update");
assert.equal(resolveAnalyseEnterAction(true, true, "add"), "add");

console.log("Analyse Enter state-machine checks passed.");
