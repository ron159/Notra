import { createContext } from "@dprint/formatter";
import dockerfilePluginUrl from "@dprint/dockerfile/plugin.wasm?url";
import jsonPluginUrl from "@dprint/json/plugin.wasm?url";
import markdownPluginUrl from "@dprint/markdown/plugin.wasm?url";
import ruffPluginUrl from "@dprint/ruff/plugin.wasm?url";
import tomlPluginUrl from "@dprint/toml/plugin.wasm?url";
import typescriptPluginUrl from "@dprint/typescript/plugin.wasm?url";

type FormatRequest = {
  id: number;
  language: string;
  filePath: string;
  source: string;
  tabSize: number;
  useTabs: boolean;
};

type FormatResponse = {
  id: number;
  text?: string;
  error?: string;
};

const pluginUrls: Record<string, string> = {
  dockerfile: dockerfilePluginUrl,
  javascript: typescriptPluginUrl,
  json: jsonPluginUrl,
  markdown: markdownPluginUrl,
  python: ruffPluginUrl,
  toml: tomlPluginUrl,
  typescript: typescriptPluginUrl,
};

const modulePromises = new Map<string, Promise<WebAssembly.Module>>();

self.addEventListener("message", async (event: MessageEvent<FormatRequest>) => {
  const request = event.data;
  const response: FormatResponse = { id: request.id };
  try {
    const pluginUrl = pluginUrls[request.language];
    if (!pluginUrl) throw new Error(`没有 ${request.language} 对应的格式化插件`);

    const context = createContext({
      indentWidth: request.tabSize,
      lineWidth: 100,
      newLineKind: "lf",
      useTabs: request.useTabs,
    });
    context.addPlugin(await loadPlugin(pluginUrl));
    response.text = context.formatText({
      filePath: request.filePath,
      fileText: request.source,
    });
  } catch (error) {
    response.error = error instanceof Error ? error.message : String(error);
  }
  self.postMessage(response);
});

function loadPlugin(url: string) {
  let promise = modulePromises.get(url);
  if (!promise) {
    promise = fetch(url).then(async (response) => {
      if (!response.ok) throw new Error(`格式化插件加载失败：HTTP ${response.status}`);
      if (typeof WebAssembly.compileStreaming === "function") {
        try {
          return await WebAssembly.compileStreaming(Promise.resolve(response.clone()));
        } catch {
          // Some WebViews do not preserve the application/wasm content type.
        }
      }
      return WebAssembly.compile(await response.arrayBuffer());
    });
    modulePromises.set(url, promise);
  }
  return promise;
}
