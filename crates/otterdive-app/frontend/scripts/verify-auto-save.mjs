import assert from "node:assert/strict";
import fs from "node:fs";

const htmlSource = fs.readFileSync(new URL("../index.html", import.meta.url), "utf8");
const mainSource = fs.readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
const stylesSource = fs.readFileSync(new URL("../src/styles.css", import.meta.url), "utf8");

assert.match(htmlSource, /id="settingsAutoSaveDelayInput"[\s\S]*min="1"[\s\S]*max="3600"[\s\S]*value="5"/);
assert.match(mainSource, /const DEFAULT_AUTO_SAVE_DELAY_SECONDS = 5;/);
assert.match(mainSource, /model\.onDidChangeContent\(\(\) => \{[\s\S]*scheduleAutoSave\(doc\);[\s\S]*scheduleSessionSave\(\);/);
assert.match(mainSource, /function scheduleAutoSave\(doc: OpenDocument\)[\s\S]*!doc\.dirty \|\| doc\.readOnly \|\| !doc\.path[\s\S]*state\.autoSaveDelaySeconds \* 1000/);
assert.match(mainSource, /function confirmDocumentCanClose[\s\S]*cancelAutoSave\(doc\.id\);[\s\S]*askUnsavedChoice/);
assert.match(mainSource, /state\.autoSaveDelaySeconds = normalizeAutoSaveDelaySeconds\(snapshot\.autoSaveDelaySeconds\);/);
assert.match(mainSource, /autoSaveDelaySeconds: state\.autoSaveDelaySeconds,/);
assert.match(stylesSource, /\.settings-number-control input[^{]*\{[^}]*text-align: right;/s);

console.log("Auto-save timeout settings and scheduling checks passed.");
