export type RegisteredLanguage = {
  id: string;
  extensions?: string[];
  filenames?: string[];
};

export type LanguageEntry = readonly [id: string, label: string, hint: string];

export const PINNED_LANGUAGES: LanguageEntry[] = [
  ["plaintext", "Plain Text", "txt"],
  ["markdown", "Markdown", "md"],
  ["mdx", "MDX", "mdx"],
  ["json", "JSON", "json, jsonc"],
  ["toml", "TOML", "toml"],
  ["yaml", "YAML", "yaml, yml"],
  ["sql", "SQL", "sql"],
  ["powershell", "PowerShell", "ps1"],
  ["javascript", "JavaScript", "js, jsx"],
  ["typescript", "TypeScript", "ts, tsx"],
  ["python", "Python", "py"],
  ["xml", "XML", "xml"],
  ["html", "HTML", "html"],
  ["css", "CSS", "css"],
  ["java", "Java", "java"],
  ["rust", "Rust", "rs"],
];

export function languageWithOverride(
  detectedLanguage: string | null | undefined,
  languageOverride: string | null | undefined,
) {
  return languageOverride || detectedLanguage || "plaintext";
}

const LANGUAGE_BY_FILE_NAME: Record<string, string> = {
  "containerfile": "dockerfile",
  "dockerfile": "dockerfile",
  "gnumakefile": "shell",
  "makefile": "shell",
  ".babelrc": "json",
  ".bowerrc": "json",
  ".eslintrc": "json",
  ".jscsrc": "json",
  ".jshintrc": "json",
  ".prettierrc": "json",
  ".dockerignore": "plaintext",
  ".env": "plaintext",
  ".env.local": "plaintext",
  ".gitignore": "plaintext",
  ".npmrc": "plaintext",
};

const LANGUAGE_BY_EXTENSION: Record<string, string> = {
  txt: "plaintext", text: "plaintext", log: "plaintext", csv: "plaintext", tsv: "plaintext",
  md: "markdown", markdown: "markdown", rmd: "markdown", mdx: "mdx",
  json: "json", jsonc: "json", har: "json", toml: "toml", yaml: "yaml", yml: "yaml",
  sql: "sql", mysql: "mysql", pgsql: "pgsql",
  ps1: "powershell", psm1: "powershell", psd1: "powershell",
  js: "javascript", jsx: "javascript", mjs: "javascript", cjs: "javascript",
  ts: "typescript", tsx: "typescript", mts: "typescript", cts: "typescript",
  py: "python", pyw: "python", pyi: "python",
  xml: "xml", xsd: "xml", xsl: "xml", svg: "xml",
  html: "html", htm: "html", xhtml: "html", css: "css", scss: "scss", less: "less",
  java: "java", rs: "rust", go: "go",
  c: "cpp", h: "cpp", cc: "cpp", cpp: "cpp", cxx: "cpp", hh: "cpp", hpp: "cpp", hxx: "cpp",
  cs: "csharp", csx: "csharp", php: "php", phtml: "php", rb: "ruby", rake: "ruby", gemspec: "ruby",
  sh: "shell", bash: "shell", zsh: "shell", fish: "shell", ksh: "shell", bat: "bat", cmd: "bat",
  ini: "ini", cfg: "ini", conf: "ini", editorconfig: "ini", properties: "ini",
  kt: "kotlin", kts: "kotlin", swift: "swift", scala: "scala", sc: "scala", dart: "dart",
  lua: "lua", pl: "perl", pm: "perl", r: "r", ex: "elixir", exs: "elixir",
  fs: "fsharp", fsi: "fsharp", fsx: "fsharp", clj: "clojure", cljs: "clojure", cljc: "clojure", edn: "clojure",
  coffee: "coffee", graphql: "graphql", gql: "graphql", tf: "hcl", tfvars: "hcl", hcl: "hcl",
  proto: "protobuf", sol: "solidity", sv: "systemverilog", svh: "systemverilog", vb: "vb", vbs: "vb",
  m: "objective-c", mm: "objective-c", pas: "pascal", pp: "pascal", pug: "pug", jade: "pug",
  hbs: "handlebars", handlebars: "handlebars", twig: "twig", liquid: "liquid", ftl: "freemarker2",
  cshtml: "razor", razor: "razor", redis: "redis", rst: "restructuredtext", rq: "sparql", sparql: "sparql",
  tcl: "tcl", wgsl: "wgsl", bicep: "bicep", apex: "apex", cls: "apex", trigger: "apex",
  abap: "abap", azcli: "azcli", cypher: "cypher", cql: "cypher", qs: "qsharp", pq: "powerquery",
  tsp: "typespec", ecl: "ecl", jl: "julia", asm: "mips", s: "mips", mips: "mips",
  mligo: "cameligo", ligo: "cameligo",
};

export function languageFromFilePath(path: string, registeredLanguages: RegisteredLanguage[]) {
  const name = fileNameFromPath(path).toLowerCase();
  const exact = LANGUAGE_BY_FILE_NAME[name];
  if (exact) return exact;

  const extension = fileExtension(name);
  const mapped = LANGUAGE_BY_EXTENSION[extension];
  if (mapped) return mapped;

  const extensionWithDot = extension ? `.${extension}` : "";
  const registered = registeredLanguages.find((language) =>
    language.filenames?.some((candidate) => candidate.toLowerCase() === name)
    || language.extensions?.some((candidate) => candidate.toLowerCase() === extensionWithDot)
  );
  return registered?.id ?? "plaintext";
}

function fileNameFromPath(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}

function fileExtension(name: string) {
  const index = name.lastIndexOf(".");
  return index > 0 ? name.slice(index + 1) : "";
}
