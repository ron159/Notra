import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const options = parseOptions(process.argv.slice(2));
const output = path.resolve(options.output);
fs.mkdirSync(output, { recursive: true });

const extension = options.kind === "markdown" ? "md" : "txt";
const row = options.kind === "markdown"
  ? "## OtterDive performance heading\n\n- deterministic fixture content\n\n"
  : "OtterDive deterministic multi-file performance fixture\n";
const targetBytes = options.sizeKb * 1024;
const repetitions = Math.ceil(targetBytes / Buffer.byteLength(row));
const content = row.repeat(repetitions).slice(0, targetBytes);

for (let index = 0; index < options.count; index += 1) {
  const name = `fixture-${String(index + 1).padStart(3, "0")}.${extension}`;
  fs.writeFileSync(path.join(output, name), content);
}

process.stdout.write(`${output}\n${options.count} files, ${options.sizeKb} KiB each, ${options.kind}\n`);

function parseOptions(args) {
  const values = new Map();
  for (let index = 0; index < args.length; index += 2) {
    values.set(args[index], args[index + 1]);
  }
  const count = positiveInteger(values.get("--count") ?? "50", "--count");
  const sizeKb = positiveInteger(values.get("--size-kb") ?? "256", "--size-kb");
  const kind = values.get("--kind") ?? "text";
  if (kind !== "text" && kind !== "markdown") {
    throw new Error("--kind must be text or markdown");
  }
  return {
    count,
    kind,
    output: values.get("--output") ?? path.join(os.tmpdir(), `otterdive-open-${count}-${sizeKb}-${kind}`),
    sizeKb,
  };
}

function positiveInteger(value, option) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) throw new Error(`${option} must be a positive integer`);
  return parsed;
}
