export const DPRINT_LANGUAGES = new Set([
  "dockerfile",
  "javascript",
  "json",
  "markdown",
  "python",
  "toml",
  "typescript",
]);

const EXTENSIONS_BY_LANGUAGE: Record<string, string[]> = {
  javascript: ["js", "jsx", "mjs", "cjs"],
  json: ["json", "jsonc"],
  markdown: ["md", "markdown"],
  python: ["py", "pyw", "pyi"],
  toml: ["toml"],
  typescript: ["ts", "tsx", "mts", "cts"],
};

const DEFAULT_FILE_BY_LANGUAGE: Record<string, string> = {
  dockerfile: "Dockerfile",
  javascript: "untitled.js",
  json: "untitled.json",
  markdown: "untitled.md",
  python: "untitled.py",
  toml: "untitled.toml",
  typescript: "untitled.ts",
};

export function supportsDprintLanguage(language: string) {
  return DPRINT_LANGUAGES.has(language);
}

export function formatterFilePath(language: string, path: string | null | undefined) {
  if (language === "dockerfile") return "Dockerfile";
  const name = path?.split(/[\\/]/).pop() ?? "";
  const extension = name.includes(".") ? name.slice(name.lastIndexOf(".") + 1).toLowerCase() : "";
  if (name && EXTENSIONS_BY_LANGUAGE[language]?.includes(extension)) return name;
  return DEFAULT_FILE_BY_LANGUAGE[language] ?? "untitled.txt";
}

export function preserveTrailingNewline(formatted: string, original: string) {
  const originalHasTrailingNewline = /(?:\r\n|\r|\n)$/.test(original);
  const withoutTrailingNewlines = formatted.replace(/(?:\r\n|\r|\n)+$/, "");
  if (!originalHasTrailingNewline) return withoutTrailingNewlines;

  const originalNewline = original.endsWith("\r\n") ? "\r\n" : original.endsWith("\r") ? "\r" : "\n";
  return `${withoutTrailingNewlines}${originalNewline}`;
}
