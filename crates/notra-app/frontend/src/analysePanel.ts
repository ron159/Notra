import { invoke } from "@tauri-apps/api/core";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import {
  resolveAnalyseEnterAction,
  type AnalyseEnterAction,
} from "./analyseState";

type AnalyseSearchType = "normal" | "escaped" | "regex" | "regexMultiline";
type AnalyseSelection = "line" | "text";
export type AnalyseProfileLoadMode = "replace" | "append" | "prepend";

export interface AnalysePanelSettings {
  autoUpdate: boolean;
  showLineNumbers: boolean;
  wordWrap: boolean;
  fontSize: number;
  scrollSync: boolean;
  boundResultPath: string | null;
  enterAction: AnalyseEnterAction;
  workingPatterns: AnalysePattern[];
}

export interface AnalyseDocumentSnapshot {
  id: number;
  revision: number;
  text: string;
  title: string;
  path: string | null;
  fileSize: number;
  dirty: boolean;
  largeFile: boolean;
}

export interface AnalysePanelHost {
  getDocument: () => AnalyseDocumentSnapshot;
  getSelectedText: () => string;
  getSourceLine: () => number;
  navigate: (documentId: number, line: number) => void;
  revealSource: (documentId: number, line: number) => void;
  setBookmarkLines: (documentId: number, lines: number[]) => void;
  getRecentProfilePaths: () => string[];
  rememberProfilePath: (path: string) => void;
  getDefaultDirectory: () => string | null;
  getSettings: () => AnalysePanelSettings;
  updateSettings: (settings: AnalysePanelSettings) => void;
  log: (message: string) => void;
}

export interface AnalysePanelController {
  addSelectionAsPattern: (text: string) => void;
  cancel: () => void;
  clearPatterns: () => void;
  focusOptions: () => void;
  focusResult: () => void;
  layout: () => void;
  loadProfile: () => Promise<void>;
  loadProfilePath: (path: string, mode?: AnalyseProfileLoadMode) => Promise<void>;
  notifyDocumentChanged: (documentId: number) => void;
  run: () => Promise<void>;
  saveProfile: () => Promise<void>;
  syncDocument: () => void;
  syncRecentProfiles: () => void;
  syncSettings: () => void;
  syncSourceLine: (line: number) => void;
}

export interface AnalysePattern {
  id: number;
  orderNum: string;
  enabled: boolean;
  searchText: string;
  searchType: AnalyseSearchType;
  matchCase: boolean;
  wholeWord: boolean;
  selection: AnalyseSelection;
  hide: boolean;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  foreground: string;
  background: string;
  comment: string;
  group: string;
}

interface AnalyseRunResponse {
  runId: number;
  documentId: number;
  documentRevision: number;
  patternRevision: number;
  lines: AnalyseLine[];
  totalMatches: number;
  patternHits: Array<{ patternId: number; hits: number }>;
  patternErrors: Array<{ patternId: number; kind: string; message: string }>;
  totalLines: number;
  resultToken?: string | null;
}

interface AnalyseResultChunk {
  lines: AnalyseLine[];
  nextOffset: number;
  done: boolean;
}

interface AnalyseLine {
  sourceLine: number;
  text: string;
  matchingPatternIds: number[];
  styledSegments: StyledSegment[];
}

interface StyledSegment {
  startByteInLine: number;
  endByteInLine: number;
  patternId?: number | null;
  hidden: boolean;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  foreground: string;
  background: string;
}

interface ResultLineMapping {
  resultLine: number;
  sourceDocumentId: number;
  sourceLine: number;
  matchingPatternIds: number[];
}

interface ParsedAnalyseProfile {
  patterns: AnalysePattern[];
  nextPatternId: number;
}

interface AnalyseResultFindMatch {
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
}

const DEFAULT_FOREGROUND = "#D32F2F";
const DEFAULT_BACKGROUND = "#FFFFFF";

export function createAnalysePanel(
  container: HTMLElement,
  host: AnalysePanelHost,
): AnalysePanelController {
  container.innerHTML = panelMarkup();
  const element = <T extends HTMLElement>(role: string) => {
    const found = container.querySelector<T>(`[data-analyse-role="${role}"]`);
    if (!found) throw new Error(`Missing Analyse control: ${role}`);
    return found;
  };

  const patterns: AnalysePattern[] = [];
  const hits = new Map<number, number>();
  const errors = new Map<number, string>();
  let selectedPatternId: number | null = null;
  let nextPatternId = 1;
  let patternRevision = 0;
  let nextRunId = 0;
  let resultDocumentId: number | null = null;
  let latestResult: AnalyseRunResponse | null = null;
  let resultMapping: ResultLineMapping[] = [];
  let currentProfilePath: string | null = null;
  let boundResultPath: string | null = null;
  let activeRunId: number | null = null;
  let autoRunTimer = 0;
  let resultFindMatches: monaco.Range[] = [];
  let activeResultFindIndex = -1;
  let resultFindSignature = "";
  let syncingFromSource = false;
  let syncingFromResult = false;

  const resultModel = monaco.editor.createModel(
    "",
    "plaintext",
    monaco.Uri.parse(`notra://analyse/result-${Date.now()}`),
  );
  const resultEditor = monaco.editor.create(element("result-editor"), {
    model: resultModel,
    readOnly: true,
    domReadOnly: true,
    minimap: { enabled: false },
    lineNumbers: "off",
    glyphMargin: false,
    folding: false,
    renderLineHighlight: "line",
    scrollBeyondLastLine: false,
    wordWrap: "off",
    automaticLayout: true,
    largeFileOptimizations: true,
    fontSize: 12,
    lineHeight: 19,
    padding: { top: 6, bottom: 6 },
    scrollbar: { verticalScrollbarSize: 9, horizontalScrollbarSize: 9 },
  });
  const resultDecorations = resultEditor.createDecorationsCollection();
  const resultFindDecorations = resultEditor.createDecorationsCollection();
  const styleElement = document.createElement("style");
  styleElement.dataset.analyseStyles = "true";
  container.appendChild(styleElement);

  writeDraft(defaultPattern(0));
  applySettings();
  bindActions();
  renderPatterns();
  renderRecentProfiles();
  setStatus("添加 Pattern 后即可分析当前文档");

  function bindActions() {
    element("run").addEventListener("click", () => void run());
    element("cancel").addEventListener("click", () => cancelActiveRun());
    element("clear-result").addEventListener("click", clearResultState);
    element("add").addEventListener("click", addPattern);
    element("add-selection").addEventListener("click", () => {
      addSelectionAsPattern(host.getSelectedText());
    });
    element("update").addEventListener("click", updatePattern);
    element("delete").addEventListener("click", deletePattern);
    element("clear-patterns").addEventListener("click", clearPatterns);
    element("enable-all").addEventListener("click", () => setAllPatternsEnabled(true));
    element("disable-all").addEventListener("click", () => setAllPatternsEnabled(false));
    element("enable-group").addEventListener("click", () => setSelectedGroupEnabled(true));
    element("disable-group").addEventListener("click", () => setSelectedGroupEnabled(false));
    element("up").addEventListener("click", () => movePattern(-1));
    element("down").addEventListener("click", () => movePattern(1));
    element("sort").addEventListener("click", sortPatterns);
    element("load-profile").addEventListener("click", () => void loadProfile());
    element("save-profile").addEventListener("click", () => void saveProfile());
    element("recent-profile-open").addEventListener("click", () => {
      const path = select("recent-profile").value;
      if (path) void loadProfilePath(path, selectedProfileLoadMode());
    });
    element("show-lines").addEventListener("change", () => {
      renderResult();
      persistSettings();
      void writeBoundResult();
    });
    element("auto-update").addEventListener("change", () => {
      persistSettings();
      scheduleAutoRun();
    });
    element("scroll-sync").addEventListener("change", () => {
      persistSettings();
      if (input("scroll-sync").checked) syncSourceLine(host.getSourceLine());
    });
    element("enter-action").addEventListener("change", persistSettings);
    element("copy-result").addEventListener("click", () => void copyResult(false));
    element("copy-result-rtf").addEventListener("click", () => void copyResult(true));
    element("save-result").addEventListener("click", () => void saveResult(false));
    element("save-result-rtf").addEventListener("click", () => void saveResult(true));
    element("bind-result").addEventListener("click", () => void bindResult());
    element("unbind-result").addEventListener("click", unbindResult);
    element("result-find-next").addEventListener("click", () => void findResult(1));
    element("result-find-previous").addEventListener("click", () => void findResult(-1));
    element("result-find-input").addEventListener("keydown", (event) => {
      const keyboardEvent = event as KeyboardEvent;
      if (keyboardEvent.key !== "Enter") return;
      keyboardEvent.preventDefault();
      void findResult(keyboardEvent.shiftKey ? -1 : 1);
    });
    element("word-wrap").addEventListener("change", () => {
      resultEditor.updateOptions({
        wordWrap: input("word-wrap").checked ? "on" : "off",
      });
      persistSettings();
    });
    element("result-font").addEventListener("change", () => {
      const fontSize = Math.min(24, Math.max(10, Number(input("result-font").value) || 12));
      input("result-font").value = String(fontSize);
      resultEditor.updateOptions({ fontSize, lineHeight: Math.round(fontSize * 1.6) });
      persistSettings();
    });
    element("pattern-list").addEventListener("click", (event) => {
      const row = (event.target as HTMLElement).closest<HTMLElement>("[data-pattern-id]");
      if (row) selectPattern(Number(row.dataset.patternId));
    });
    element("pattern-list").addEventListener("dblclick", (event) => {
      const row = (event.target as HTMLElement).closest<HTMLElement>("[data-pattern-id]");
      if (!row) return;
      const pattern = patterns.find((item) => item.id === Number(row.dataset.patternId));
      if (!pattern) return;
      pattern.enabled = !pattern.enabled;
      invalidateResult(false);
      if (pattern.id === selectedPatternId) writeDraft(pattern);
      renderPatterns();
      persistSettings();
      void run();
    });
    element("pattern-list").addEventListener("keydown", (event) => {
      const keyboardEvent = event as KeyboardEvent;
      if (keyboardEvent.key !== "ArrowDown" && keyboardEvent.key !== "ArrowUp") return;
      keyboardEvent.preventDefault();
      const index = Math.max(0, patterns.findIndex((item) => item.id === selectedPatternId));
      const next = Math.min(
        patterns.length - 1,
        Math.max(0, index + (keyboardEvent.key === "ArrowDown" ? 1 : -1)),
      );
      if (patterns[next]) selectPattern(patterns[next].id, true);
    });
    element("search-text").addEventListener("keydown", (event) => {
      const keyboardEvent = event as KeyboardEvent;
      if (keyboardEvent.key === "Escape") {
        keyboardEvent.preventDefault();
        const selected = patterns.find((item) => item.id === selectedPatternId);
        writeDraft(selected ?? defaultPattern(0));
        setStatus("已放弃未提交的 Pattern 修改");
        return;
      }
      if (keyboardEvent.key === "Enter") {
        keyboardEvent.preventDefault();
        handleEnterAction();
      }
    });
    resultEditor.onMouseDown((event) => {
      if (event.event.detail < 2 || !event.target.position) return;
      const mapping = resultMapping[event.target.position.lineNumber - 1];
      if (mapping) host.navigate(mapping.sourceDocumentId, mapping.sourceLine);
    });
    resultEditor.onDidChangeCursorPosition(({ position }) => {
      renderMatchingPatterns(position.lineNumber);
    });
    resultEditor.onDidScrollChange(() => {
      if (!input("scroll-sync").checked || syncingFromSource || !latestResult) return;
      const lineNumber = resultEditor.getVisibleRanges()[0]?.startLineNumber;
      const mapping = lineNumber ? resultMapping[lineNumber - 1] : undefined;
      if (!mapping) return;
      syncingFromResult = true;
      host.revealSource(mapping.sourceDocumentId, mapping.sourceLine);
      queueMicrotask(() => { syncingFromResult = false; });
    });
  }

  function input(role: string) {
    return element<HTMLInputElement>(role);
  }

  function select(role: string) {
    return element<HTMLSelectElement>(role);
  }

  function readDraft(id: number): AnalysePattern {
    return {
      id,
      orderNum: input("order").value.trim(),
      enabled: input("enabled").checked,
      searchText: input("search-text").value,
      searchType: select("search-type").value as AnalyseSearchType,
      matchCase: input("match-case").checked,
      wholeWord: input("whole-word").checked,
      selection: select("selection").value as AnalyseSelection,
      hide: input("hide").checked,
      bold: input("bold").checked,
      italic: input("italic").checked,
      underline: input("underline").checked,
      foreground: input("foreground").value.toUpperCase(),
      background: input("background").value.toUpperCase(),
      comment: input("comment").value,
      group: input("group").value,
    };
  }

  function writeDraft(pattern: AnalysePattern) {
    input("order").value = pattern.orderNum;
    input("enabled").checked = pattern.enabled;
    input("search-text").value = pattern.searchText;
    select("search-type").value = pattern.searchType;
    input("match-case").checked = pattern.matchCase;
    input("whole-word").checked = pattern.wholeWord;
    select("selection").value = pattern.selection;
    input("hide").checked = pattern.hide;
    input("bold").checked = pattern.bold;
    input("italic").checked = pattern.italic;
    input("underline").checked = pattern.underline;
    input("foreground").value = pattern.foreground;
    input("background").value = pattern.background;
    input("comment").value = pattern.comment;
    input("group").value = pattern.group;
  }

  function draftChanged() {
    const current = patterns.find((item) => item.id === selectedPatternId);
    return current ? JSON.stringify(current) !== JSON.stringify(readDraft(current.id)) : true;
  }

  function addPattern() {
    const pattern = readDraft(nextPatternId++);
    if (!pattern.searchText) {
      setStatus("Search Text 不能为空", true);
      input("search-text").focus();
      return;
    }
    const selectedIndex = patterns.findIndex((item) => item.id === selectedPatternId);
    patterns.splice(selectedIndex < 0 ? patterns.length : selectedIndex + 1, 0, pattern);
    selectedPatternId = pattern.id;
    invalidateResult();
    renderPatterns();
    persistSettings();
    setStatus(`已添加 Pattern ${pattern.id}`);
  }

  function addSelectionAsPattern(text: string) {
    if (!text) {
      setStatus("请先在源编辑器中选择文本", true);
      return;
    }
    const pattern = readDraft(nextPatternId++);
    pattern.searchText = text;
    const selectedIndex = patterns.findIndex((item) => item.id === selectedPatternId);
    patterns.splice(selectedIndex < 0 ? patterns.length : selectedIndex + 1, 0, pattern);
    selectedPatternId = pattern.id;
    writeDraft(pattern);
    invalidateResult();
    renderPatterns();
    persistSettings();
    setStatus(`已从源选区添加 Pattern ${pattern.id}`);
  }

  function updatePattern() {
    const index = patterns.findIndex((item) => item.id === selectedPatternId);
    if (index < 0) {
      addPattern();
      return;
    }
    const updated = readDraft(patterns[index].id);
    if (!updated.searchText) {
      setStatus("Search Text 不能为空", true);
      return;
    }
    patterns[index] = updated;
    invalidateResult();
    renderPatterns();
    persistSettings();
    setStatus(`已更新 Pattern ${updated.id}`);
  }

  function deletePattern() {
    const index = patterns.findIndex((item) => item.id === selectedPatternId);
    if (index < 0) return;
    const [removed] = patterns.splice(index, 1);
    hits.delete(removed.id);
    errors.delete(removed.id);
    selectedPatternId = patterns[Math.min(index, patterns.length - 1)]?.id ?? null;
    invalidateResult(false);
    if (selectedPatternId !== null) {
      const selected = patterns.find((item) => item.id === selectedPatternId);
      if (selected) writeDraft(selected);
    } else {
      writeDraft(defaultPattern(0));
    }
    renderPatterns();
    persistSettings();
    scheduleAutoRun();
  }

  function movePattern(delta: number) {
    const index = patterns.findIndex((item) => item.id === selectedPatternId);
    const target = index + delta;
    if (index < 0 || target < 0 || target >= patterns.length) return;
    [patterns[index], patterns[target]] = [patterns[target], patterns[index]];
    invalidateResult();
    renderPatterns();
    persistSettings();
    focusSelectedRow();
  }

  function sortPatterns() {
    patterns.sort((left, right) => compareOrder(left.orderNum, right.orderNum));
    invalidateResult();
    renderPatterns();
    persistSettings();
    focusSelectedRow();
  }

  function handleEnterAction() {
    const action = resolveAnalyseEnterAction(
      selectedPatternId !== null,
      draftChanged(),
      select("enter-action").value as AnalyseEnterAction,
    );
    if (action === "update") updatePattern();
    else if (action === "add") addPattern();
    void run();
  }

  function clearPatterns() {
    if (patterns.length === 0) {
      setStatus("Pattern 列表已经为空");
      return;
    }
    patterns.splice(0, patterns.length);
    selectedPatternId = null;
    nextPatternId = 1;
    writeDraft(defaultPattern(0));
    invalidateResult(false);
    renderPatterns();
    persistSettings();
    setStatus("已清空全部 Pattern");
  }

  function setAllPatternsEnabled(enabled: boolean) {
    setPatternsEnabled(patterns, enabled, enabled ? "已启用全部 Pattern" : "已禁用全部 Pattern");
  }

  function setSelectedGroupEnabled(enabled: boolean) {
    const group = (
      patterns.find((item) => item.id === selectedPatternId)?.group
      ?? input("group").value
    ).trim();
    if (!group) {
      setStatus("请先选择带 Group 的 Pattern", true);
      return;
    }
    setPatternsEnabled(
      patterns.filter((pattern) => pattern.group === group),
      enabled,
      `${enabled ? "已启用" : "已禁用"} Group：${group}`,
    );
  }

  function setPatternsEnabled(targets: AnalysePattern[], enabled: boolean, message: string) {
    if (targets.length === 0) {
      setStatus("没有符合条件的 Pattern", true);
      return;
    }
    let changed = false;
    for (const pattern of targets) {
      if (pattern.enabled === enabled) continue;
      pattern.enabled = enabled;
      changed = true;
    }
    if (!changed) {
      setStatus(message);
      return;
    }
    const selected = patterns.find((item) => item.id === selectedPatternId);
    if (selected) writeDraft(selected);
    invalidateResult();
    renderPatterns();
    persistSettings();
    setStatus(message);
  }

  function selectPattern(id: number, focus = false) {
    const pattern = patterns.find((item) => item.id === id);
    if (!pattern) return;
    selectedPatternId = id;
    writeDraft(pattern);
    renderPatterns();
    if (focus) focusSelectedRow();
  }

  function focusSelectedRow() {
    container
      .querySelector<HTMLElement>(`[data-pattern-id="${selectedPatternId}"]`)
      ?.focus();
  }

  function renderPatterns() {
    const body = element<HTMLTableSectionElement>("pattern-list");
    body.replaceChildren();
    for (const [index, pattern] of patterns.entries()) {
      const row = document.createElement("tr");
      row.tabIndex = 0;
      row.dataset.patternId = String(pattern.id);
      row.classList.toggle("selected", pattern.id === selectedPatternId);
      row.classList.toggle("disabled", !pattern.enabled);
      const error = errors.get(pattern.id);
      const values = [
        pattern.enabled ? "●" : "○",
        pattern.orderNum || String(index + 1),
        pattern.searchText,
        searchTypeLabel(pattern.searchType),
        pattern.group || "—",
        String(hits.get(pattern.id) ?? 0),
        error ? "错误" : "就绪",
      ];
      for (const [cellIndex, value] of values.entries()) {
        const cell = document.createElement("td");
        cell.textContent = value;
        if (cellIndex === 2) cell.title = pattern.searchText;
        if (cellIndex === 6 && error) {
          cell.className = "analyse-pattern-error";
          cell.title = error;
        }
        row.appendChild(cell);
      }
      body.appendChild(row);
    }
    element("pattern-count").textContent = `${patterns.length} 个 Pattern`;
    element<HTMLButtonElement>("update").disabled = selectedPatternId === null;
    element<HTMLButtonElement>("delete").disabled = selectedPatternId === null;
    element<HTMLButtonElement>("up").disabled = selectedPatternId === null;
    element<HTMLButtonElement>("down").disabled = selectedPatternId === null;
  }

  async function loadProfile() {
    try {
      const path = await invoke<string | null>("pick_file_path", {
        request: { defaultDir: host.getDefaultDirectory() },
      });
      if (path) await loadProfilePath(path, selectedProfileLoadMode());
    } catch (error) {
      reportProfileError("加载 Profile 失败", error);
    }
  }

  async function loadProfilePath(path: string, mode: AnalyseProfileLoadMode = "replace") {
    try {
      const document = await invoke<{ text: string }>("open_path", { path });
      const firstPatternId = mode === "replace" ? 1 : nextPatternId;
      const parsed = await invoke<ParsedAnalyseProfile>("parse_analyse_profile", {
        request: { xml: document.text, firstPatternId },
      });
      if (mode === "replace") patterns.splice(0, patterns.length, ...parsed.patterns);
      else if (mode === "prepend") patterns.unshift(...parsed.patterns);
      else patterns.push(...parsed.patterns);
      nextPatternId = parsed.nextPatternId;
      selectedPatternId = parsed.patterns[0]?.id ?? patterns[0]?.id ?? null;
      const selectedPattern = patterns.find((pattern) => pattern.id === selectedPatternId);
      writeDraft(selectedPattern ?? defaultPattern(0));
      currentProfilePath = path;
      host.rememberProfilePath(path);
      renderRecentProfiles();
      invalidateResult();
      renderPatterns();
      persistSettings();
      setStatus(`${profileModeLabel(mode)} ${fileName(path)}：${parsed.patterns.length} 个 Pattern`);
    } catch (error) {
      reportProfileError(`加载 Profile 失败：${fileName(path)}`, error);
    }
  }

  async function saveProfile() {
    try {
      const xml = await invoke<string>("write_analyse_profile", {
        request: {
          patterns,
          patternHits: [...hits].map(([patternId, patternHits]) => ({ patternId, hits: patternHits })),
        },
      });
      const path = await invoke<string | null>("pick_save_path", {
        request: {
          defaultDir: currentProfilePath ? pathDirectory(currentProfilePath) : host.getDefaultDirectory(),
          fileName: currentProfilePath ? fileName(currentProfilePath) : "Analyse.xml",
        },
      });
      if (!path) return;
      await invoke("save_document", {
        request: { path, text: xml, encoding: "UTF-8", lineEnding: "LF" },
      });
      currentProfilePath = path;
      host.rememberProfilePath(path);
      renderRecentProfiles();
      setStatus(`已保存 Profile：${fileName(path)}`);
    } catch (error) {
      reportProfileError("保存 Profile 失败", error);
    }
  }

  function renderRecentProfiles() {
    const recent = select("recent-profile");
    const selectedPath = recent.value || currentProfilePath || "";
    recent.replaceChildren(new Option("最近 Profile", ""));
    for (const path of host.getRecentProfilePaths()) {
      const option = new Option(fileName(path), path);
      option.title = path;
      recent.appendChild(option);
    }
    recent.value = host.getRecentProfilePaths().includes(selectedPath) ? selectedPath : "";
    element<HTMLButtonElement>("recent-profile-open").disabled = recent.options.length <= 1;
  }

  function selectedProfileLoadMode() {
    return select("profile-load-mode").value as AnalyseProfileLoadMode;
  }

  function reportProfileError(message: string, error: unknown) {
    const detail = `${message}：${String(error)}`;
    setStatus(detail, true);
    host.log(detail);
  }

  async function run() {
    if (patterns.length === 0) {
      clearResult();
      setStatus("请先添加至少一个 Pattern", true);
      return;
    }
    const documentSnapshot = host.getDocument();
    if (activeRunId !== null) cancelActiveRun(false);
    const runId = ++nextRunId;
    activeRunId = runId;
    const requestedPatternRevision = patternRevision;
    setRunning(true);
    setStatus(`正在分析 ${documentSnapshot.title}…`);
    try {
      const commonRequest = {
        runId,
        documentId: documentSnapshot.id,
        documentRevision: documentSnapshot.revision,
        patternRevision: requestedPatternRevision,
        patterns,
      };
      const usePath = documentSnapshot.largeFile
        && !documentSnapshot.dirty
        && Boolean(documentSnapshot.path);
      const result = usePath
        ? await invoke<AnalyseRunResponse>("run_analyse_path", {
          request: {
            ...commonRequest,
            path: documentSnapshot.path,
            expectedFileSize: documentSnapshot.fileSize,
          },
        })
        : await invoke<AnalyseRunResponse>("run_analyse", {
          request: { ...commonRequest, text: documentSnapshot.text },
        });
      if (!isCurrentResult(result)) {
        await releaseResultBatch(result.resultToken);
        host.log("Analyse 已丢弃过期结果");
        return;
      }
      if (!await consumeResultBatches(result)) return;
      latestResult = result;
      resultDocumentId = result.documentId;
      hits.clear();
      errors.clear();
      for (const item of result.patternHits) hits.set(item.patternId, item.hits);
      for (const item of result.patternErrors) errors.set(item.patternId, item.message);
      renderPatterns();
      renderResult();
      host.setBookmarkLines(result.documentId, result.lines.map((line) => line.sourceLine));
      await writeBoundResult();
      setStatus(
        `${result.totalLines} 行，${result.totalMatches} 个匹配${result.patternErrors.length ? `，${result.patternErrors.length} 个 Pattern 错误` : ""}`,
        result.patternErrors.length > 0,
      );
    } catch (error) {
      if (runId !== nextRunId) return;
      setStatus(`Analyse 失败：${String(error)}`, true);
      host.log(`Analyse 失败：${String(error)}`);
    } finally {
      if (activeRunId === runId) activeRunId = null;
      if (runId === nextRunId) setRunning(false);
    }
  }

  function isCurrentResult(result: AnalyseRunResponse) {
    const current = host.getDocument();
    return result.runId === nextRunId
      && result.documentId === current.id
      && result.documentRevision === current.revision
      && result.patternRevision === patternRevision;
  }

  async function consumeResultBatches(result: AnalyseRunResponse) {
    const resultToken = result.resultToken;
    if (!resultToken) return true;
    let offset = result.lines.length;
    try {
      while (offset < result.totalLines) {
        if (!isCurrentResult(result)) {
          await releaseResultBatch(resultToken);
          return false;
        }
        setStatus(`正在接收结果 ${offset}/${result.totalLines} 行…`);
        const chunk = await invoke<AnalyseResultChunk>("read_analyse_result_chunk", {
          request: { resultToken, offset, limit: 2_000 },
        });
        result.lines.push(...chunk.lines);
        offset = chunk.nextOffset;
        if (chunk.done) break;
        await new Promise<void>((resolve) => window.requestAnimationFrame(() => resolve()));
      }
      result.resultToken = null;
      return isCurrentResult(result);
    } catch (error) {
      await releaseResultBatch(resultToken);
      throw error;
    }
  }

  async function releaseResultBatch(resultToken: string | null | undefined) {
    if (!resultToken) return;
    await invoke<boolean>("release_analyse_result", { resultToken }).catch(() => false);
  }

  function renderResult() {
    if (!latestResult) {
      clearResultModel();
      return;
    }
    const showLines = input("show-lines").checked;
    const digits = String(latestResult.lines.at(-1)?.sourceLine ?? 1).length;
    const renderedLines: string[] = [];
    resultMapping = [];
    const decorations: monaco.editor.IModelDeltaDecoration[] = [];
    const styleClasses = new Map<string, string>();
    const styleRules: string[] = [];

    latestResult.lines.forEach((line, index) => {
      const prefix = showLines ? `${String(line.sourceLine).padStart(digits, " ")}: ` : "";
      renderedLines.push(prefix + line.text);
      resultMapping.push({
        resultLine: index + 1,
        sourceDocumentId: latestResult?.documentId ?? 0,
        sourceLine: line.sourceLine,
        matchingPatternIds: line.matchingPatternIds,
      });
      for (const segment of line.styledSegments) {
        if (segment.startByteInLine >= segment.endByteInLine) continue;
        const styleKey = JSON.stringify([
          segment.hidden,
          segment.bold,
          segment.italic,
          segment.underline,
          segment.foreground,
          segment.background,
        ]);
        let className = styleClasses.get(styleKey);
        if (!className) {
          className = `analyse-result-style-${styleClasses.size}`;
          styleClasses.set(styleKey, className);
          styleRules.push(styleRule(className, segment));
        }
        decorations.push({
          range: new monaco.Range(
            index + 1,
            prefix.length + utf16ColumnAtByte(line.text, segment.startByteInLine),
            index + 1,
            prefix.length + utf16ColumnAtByte(line.text, segment.endByteInLine),
          ),
          options: {
            inlineClassName: className,
            hoverMessage: segment.patternId
              ? { value: `Pattern ${segment.patternId}` }
              : undefined,
          },
        });
      }
    });
    styleElement.textContent = styleRules.join("\n");
    resultModel.setValue(renderedLines.join("\n"));
    resultDecorations.set(decorations);
    clearResultFind();
    renderMatchingPatterns(1);
    element("result-summary").textContent = `${latestResult.lines.length} 行 / ${latestResult.totalMatches} 匹配`;
  }

  async function findResult(direction: 1 | -1) {
    if (!latestResult) return;
    const query = input("result-find-input").value;
    if (!query) {
      clearResultFind();
      return;
    }
    const signature = JSON.stringify([
      latestResult.runId,
      query,
      select("result-find-type").value,
      input("result-find-case").checked,
      input("result-find-word").checked,
      input("show-lines").checked,
    ]);
    if (signature !== resultFindSignature) {
      try {
        const found = await invoke<AnalyseResultFindMatch[]>("find_analyse_result", {
          request: {
            text: latestResult.lines.map((line) => line.text).join("\n"),
            query,
            searchType: select("result-find-type").value,
            matchCase: input("result-find-case").checked,
            wholeWord: input("result-find-word").checked,
          },
        });
        if (!latestResult || signature !== JSON.stringify([
          latestResult.runId,
          input("result-find-input").value,
          select("result-find-type").value,
          input("result-find-case").checked,
          input("result-find-word").checked,
          input("show-lines").checked,
        ])) return;
        resultFindMatches = found.map((match) => new monaco.Range(
          match.startLine,
          resultPrefixLength(match.startLine) + match.startColumn,
          match.endLine,
          resultPrefixLength(match.endLine) + match.endColumn,
        ));
        resultFindSignature = signature;
        activeResultFindIndex = direction > 0 ? -1 : 0;
      } catch (error) {
        element("result-find-summary").textContent = `查找失败：${String(error)}`;
        clearResultFindDecorations();
        return;
      }
    }
    if (resultFindMatches.length === 0) {
      activeResultFindIndex = -1;
      clearResultFindDecorations();
      element("result-find-summary").textContent = "无匹配";
      return;
    }
    activeResultFindIndex = (
      activeResultFindIndex + direction + resultFindMatches.length
    ) % resultFindMatches.length;
    renderResultFindDecorations();
    const active = resultFindMatches[activeResultFindIndex];
    resultEditor.setSelection(active);
    resultEditor.revealRangeInCenterIfOutsideViewport(active);
    element("result-find-summary").textContent = `${activeResultFindIndex + 1}/${resultFindMatches.length}`;
  }

  function resultPrefixLength(_lineNumber: number) {
    if (!latestResult || !input("show-lines").checked) return 0;
    const digits = String(latestResult.lines.at(-1)?.sourceLine ?? 1).length;
    return digits + 2;
  }

  function renderResultFindDecorations() {
    resultFindDecorations.set(resultFindMatches.map((range, index) => ({
      range,
      options: {
        inlineClassName: index === activeResultFindIndex
          ? "analyse-result-find-active"
          : "analyse-result-find-match",
      },
    })));
  }

  function clearResultFindDecorations() {
    resultFindDecorations.clear();
  }

  function clearResultFind() {
    resultFindMatches = [];
    activeResultFindIndex = -1;
    resultFindSignature = "";
    clearResultFindDecorations();
    element("result-find-summary").textContent = "";
  }

  function renderMatchingPatterns(resultLine: number) {
    const mapping = resultMapping[resultLine - 1];
    if (!mapping) {
      element("matching-patterns").textContent = "当前行无匹配 Pattern";
      return;
    }
    const labels = mapping.matchingPatternIds.map((id) => {
      const pattern = patterns.find((item) => item.id === id);
      return pattern ? `#${id} ${pattern.searchText}` : `#${id}`;
    });
    element("matching-patterns").textContent = labels.length > 0
      ? `Matching Patterns：${labels.join("；")}`
      : "当前行无匹配 Pattern";
  }

  function syncSourceLine(sourceLine: number) {
    if (!input("scroll-sync").checked || syncingFromResult || resultMapping.length === 0) return;
    let closest = 0;
    for (let index = 1; index < resultMapping.length; index += 1) {
      if (
        Math.abs(resultMapping[index].sourceLine - sourceLine)
        < Math.abs(resultMapping[closest].sourceLine - sourceLine)
      ) closest = index;
    }
    syncingFromSource = true;
    resultEditor.revealLineInCenterIfOutsideViewport(closest + 1);
    queueMicrotask(() => { syncingFromSource = false; });
  }

  async function copyResult(rich: boolean) {
    if (!latestResult) {
      setStatus("当前没有可复制的 Analyse 结果", true);
      return;
    }
    try {
      const plain = resultModel.getValue();
      if (rich && typeof ClipboardItem !== "undefined" && navigator.clipboard.write) {
        try {
          const payload: Record<string, Blob> = {
            "text/plain": new Blob([plain], { type: "text/plain" }),
          };
          const formats: string[] = [];
          if (clipboardSupportsMime("text/rtf")) {
            const rtf = await serializeRtf();
            payload["text/rtf"] = new Blob([rtf], { type: "text/rtf" });
            formats.push("RTF");
          }
          if (clipboardSupportsMime("text/html")) {
            const html = await serializeHtml();
            payload["text/html"] = new Blob([html], { type: "text/html" });
            formats.push("HTML");
          }
          if (formats.length === 0) throw new Error("No rich clipboard MIME is supported");
          await navigator.clipboard.write([new ClipboardItem(payload)]);
          setStatus(`已复制 Analyse 富文本结果（${formats.join(" + ")}）`);
        } catch {
          await navigator.clipboard.writeText(plain);
          setStatus("Rich Clipboard 不可用，已复制纯文本");
        }
      } else {
        await navigator.clipboard.writeText(plain);
        setStatus(rich ? "当前平台不支持 Rich Clipboard，已复制纯文本" : "已复制 Analyse 结果");
      }
    } catch (error) {
      setStatus(`复制结果失败：${String(error)}`, true);
    }
  }

  async function saveResult(rich: boolean) {
    if (!latestResult) {
      setStatus("当前没有可保存的 Analyse 结果", true);
      return;
    }
    try {
      const path = await invoke<string | null>("pick_save_path", {
        request: {
          defaultDir: host.getDefaultDirectory(),
          fileName: rich ? "Analyse-result.rtf" : "Analyse-result.txt",
        },
      });
      if (!path) return;
      const text = rich ? await serializeRtf() : resultModel.getValue();
      await saveTextFile(path, text);
      setStatus(`已保存 Analyse ${rich ? "RTF" : "文本"}结果：${fileName(path)}`);
    } catch (error) {
      setStatus(`保存结果失败：${String(error)}`, true);
    }
  }

  async function bindResult() {
    try {
      const path = await invoke<string | null>("pick_save_path", {
        request: {
          defaultDir: boundResultPath ? pathDirectory(boundResultPath) : host.getDefaultDirectory(),
          fileName: boundResultPath ? fileName(boundResultPath) : "Analyse-bound-result.txt",
        },
      });
      if (!path) return;
      boundResultPath = path;
      persistSettings();
      renderBoundResult();
      await writeBoundResult();
      setStatus(`Result 已绑定：${fileName(path)}`);
    } catch (error) {
      setStatus(`绑定 Result 失败：${String(error)}`, true);
    }
  }

  function unbindResult() {
    if (!boundResultPath) return;
    boundResultPath = null;
    persistSettings();
    renderBoundResult();
    setStatus("Result 文件绑定已解除");
  }

  async function writeBoundResult() {
    if (!boundResultPath || !latestResult) return;
    try {
      await saveTextFile(boundResultPath, resultModel.getValue());
    } catch (error) {
      const message = `更新绑定 Result 失败：${String(error)}`;
      setStatus(message, true);
      host.log(message);
    }
  }

  function saveTextFile(path: string, text: string) {
    return invoke("save_document", {
      request: { path, text, encoding: "UTF-8", lineEnding: "LF" },
    });
  }

  function serializeRtf() {
    if (!latestResult) return Promise.reject(new Error("Analyse Result is empty"));
    return invoke<string>("serialize_analyse_rtf", {
      request: {
        lines: latestResult.lines,
        showLineNumbers: input("show-lines").checked,
        fontSize: Number(input("result-font").value) || 12,
      },
    });
  }

  function serializeHtml() {
    if (!latestResult) return Promise.reject(new Error("Analyse Result is empty"));
    return invoke<string>("serialize_analyse_html", {
      request: {
        lines: latestResult.lines,
        showLineNumbers: input("show-lines").checked,
        fontSize: Number(input("result-font").value) || 12,
      },
    });
  }

  function applySettings() {
    const settings = host.getSettings();
    input("auto-update").checked = settings.autoUpdate;
    input("show-lines").checked = settings.showLineNumbers;
    input("word-wrap").checked = settings.wordWrap;
    input("scroll-sync").checked = settings.scrollSync;
    select("enter-action").value = settings.enterAction;
    input("result-font").value = String(Math.min(24, Math.max(10, settings.fontSize || 12)));
    boundResultPath = settings.boundResultPath;
    const restoredPatterns = Array.isArray(settings.workingPatterns)
      ? settings.workingPatterns.map((pattern) => ({ ...pattern }))
      : [];
    patterns.splice(0, patterns.length, ...restoredPatterns);
    selectedPatternId = patterns[0]?.id ?? null;
    nextPatternId = Math.max(0, ...patterns.map((pattern) => pattern.id)) + 1;
    writeDraft(patterns[0] ?? defaultPattern(0));
    renderPatterns();
    resultEditor.updateOptions({
      wordWrap: settings.wordWrap ? "on" : "off",
      fontSize: Number(input("result-font").value),
      lineHeight: Math.round(Number(input("result-font").value) * 1.6),
    });
    renderBoundResult();
  }

  function persistSettings() {
    host.updateSettings({
      autoUpdate: input("auto-update").checked,
      showLineNumbers: input("show-lines").checked,
      wordWrap: input("word-wrap").checked,
      fontSize: Number(input("result-font").value) || 12,
      scrollSync: input("scroll-sync").checked,
      boundResultPath,
      enterAction: select("enter-action").value as AnalyseEnterAction,
      workingPatterns: patterns.map((pattern) => ({ ...pattern })),
    });
  }

  function renderBoundResult() {
    element("bound-result").textContent = boundResultPath
      ? `绑定：${fileName(boundResultPath)}`
      : "未绑定结果文件";
    element<HTMLButtonElement>("unbind-result").disabled = !boundResultPath;
  }

  function clearResultState() {
    if (activeRunId !== null) cancelActiveRun(false);
    else nextRunId += 1;
    latestResult = null;
    hits.clear();
    errors.clear();
    if (resultDocumentId !== null) host.setBookmarkLines(resultDocumentId, []);
    resultDocumentId = null;
    clearResult();
    renderPatterns();
    setStatus("Analyse 结果已清除");
  }

  function clearResult(updateBookmarks = false) {
    if (updateBookmarks && resultDocumentId !== null) host.setBookmarkLines(resultDocumentId, []);
    latestResult = null;
    resultDocumentId = null;
    clearResultModel();
  }

  function invalidateResult(autoRun = true) {
    patternRevision += 1;
    if (activeRunId !== null) cancelActiveRun(false);
    else nextRunId += 1;
    hits.clear();
    errors.clear();
    clearResult(true);
    if (autoRun) scheduleAutoRun();
  }

  function clearResultModel() {
    resultMapping = [];
    styleElement.textContent = "";
    resultDecorations.clear();
    clearResultFind();
    resultModel.setValue("");
    element("result-summary").textContent = "暂无结果";
    element("matching-patterns").textContent = "当前行无匹配 Pattern";
  }

  function setRunning(running: boolean) {
    element<HTMLButtonElement>("run").disabled = running;
    element("run").classList.toggle("running", running);
    element("run").textContent = running ? "分析中…" : "运行";
    element<HTMLButtonElement>("cancel").disabled = !running;
  }

  function cancelActiveRun(updateStatus = true) {
    const runId = activeRunId;
    if (runId === null) return;
    activeRunId = null;
    nextRunId += 1;
    setRunning(false);
    void invoke<boolean>("cancel_analyse", { runId }).catch(() => false);
    if (updateStatus) setStatus("Analyse 已取消");
  }

  function scheduleAutoRun() {
    window.clearTimeout(autoRunTimer);
    if (!input("auto-update").checked || patterns.length === 0) return;
    autoRunTimer = window.setTimeout(() => void run(), 300);
  }

  function notifyDocumentChanged(documentId: number) {
    if (documentId !== host.getDocument().id) return;
    if (resultDocumentId === documentId) clearResult(true);
    scheduleAutoRun();
  }

  function setStatus(message: string, error = false) {
    const status = element("status");
    status.textContent = message;
    status.classList.toggle("error", error);
  }

  function syncDocument() {
    const current = host.getDocument();
    if (resultDocumentId !== null && resultDocumentId !== current.id) {
      if (activeRunId !== null) cancelActiveRun(false);
      clearResult(true);
      setStatus(`已切换到 ${current.title}，运行以生成结果`);
      scheduleAutoRun();
    }
    resultEditor.layout();
  }

  return {
    addSelectionAsPattern,
    cancel: cancelActiveRun,
    clearPatterns,
    focusOptions: () => input("search-text").focus(),
    focusResult: () => resultEditor.focus(),
    layout: () => resultEditor.layout(),
    loadProfile,
    loadProfilePath,
    notifyDocumentChanged,
    run,
    saveProfile,
    syncDocument,
    syncRecentProfiles: renderRecentProfiles,
    syncSettings: applySettings,
    syncSourceLine,
  };
}

function defaultPattern(id: number): AnalysePattern {
  return {
    id,
    orderNum: "",
    enabled: true,
    searchText: "",
    searchType: "normal",
    matchCase: false,
    wholeWord: false,
    selection: "line",
    hide: false,
    bold: false,
    italic: false,
    underline: false,
    foreground: DEFAULT_FOREGROUND,
    background: DEFAULT_BACKGROUND,
    comment: "",
    group: "",
  };
}

function compareOrder(left: string, right: string) {
  const leftNumber = Number(left);
  const rightNumber = Number(right);
  if (left.trim() && right.trim() && Number.isFinite(leftNumber) && Number.isFinite(rightNumber)) {
    return leftNumber - rightNumber;
  }
  return left.localeCompare(right, undefined, { numeric: true });
}

function searchTypeLabel(type: AnalyseSearchType) {
  switch (type) {
    case "escaped": return "扩展";
    case "regex": return "正则";
    case "regexMultiline": return "跨行正则";
    default: return "普通";
  }
}

function utf16ColumnAtByte(text: string, byteOffset: number) {
  const encoder = new TextEncoder();
  let bytes = 0;
  let utf16Units = 0;
  for (const character of text) {
    const nextBytes = bytes + encoder.encode(character).length;
    if (nextBytes > byteOffset) break;
    bytes = nextBytes;
    utf16Units += character.length;
  }
  return utf16Units + 1;
}

function styleRule(className: string, segment: StyledSegment) {
  const foreground = validColor(segment.foreground) ? segment.foreground : "inherit";
  const background = validColor(segment.background) ? segment.background : "transparent";
  const decorations = segment.underline ? "underline" : "none";
  const hidden = segment.hidden ? "opacity:0;" : "";
  return `.${className}{color:${foreground}!important;background:${background}!important;font-weight:${segment.bold ? 700 : 400};font-style:${segment.italic ? "italic" : "normal"};text-decoration:${decorations};${hidden}}`;
}

function validColor(value: string) {
  return /^#[0-9A-F]{6}$/i.test(value);
}

function clipboardSupportsMime(mime: string) {
  const supports = (ClipboardItem as typeof ClipboardItem & {
    supports?: (type: string) => boolean;
  }).supports;
  return typeof supports === "function" ? supports.call(ClipboardItem, mime) : mime === "text/html";
}

function fileName(path: string) {
  return path.split(/[\\/]/).pop() || path;
}

function pathDirectory(path: string) {
  const index = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  if (index === 2 && path[1] === ":") return path.slice(0, 3);
  return index > 0 ? path.slice(0, index) : null;
}

function profileModeLabel(mode: AnalyseProfileLoadMode) {
  if (mode === "append") return "已追加";
  if (mode === "prepend") return "已前置";
  return "已加载";
}

function panelMarkup() {
  return `
    <div class="analyse-panel">
      <header class="analyse-panel-head">
        <div><strong>Analyse</strong><span data-analyse-role="pattern-count">0 个 Pattern</span></div>
        <div>
          <label class="analyse-auto-update"><input data-analyse-role="auto-update" type="checkbox" />自动</label>
          <button class="primary" data-analyse-role="run" type="button">运行</button>
          <button data-analyse-role="cancel" type="button" disabled>取消</button>
          <button data-analyse-role="clear-result" type="button">清除结果</button>
        </div>
      </header>
      <section class="analyse-profile-bar" aria-label="Analyse Profile">
        <select data-analyse-role="profile-load-mode" aria-label="Profile 加载方式"><option value="replace">替换</option><option value="append">追加</option><option value="prepend">前置</option></select>
        <button data-analyse-role="load-profile" type="button">加载 XML</button>
        <button data-analyse-role="save-profile" type="button">保存 XML</button>
        <select data-analyse-role="recent-profile" aria-label="最近 Profile"></select>
        <button data-analyse-role="recent-profile-open" type="button">打开最近</button>
      </section>
      <section class="analyse-draft" aria-label="Pattern 配置">
        <label class="analyse-search-text"><span>Search Text</span><input data-analyse-role="search-text" autocomplete="off" placeholder="输入文本、扩展表达式或正则" /></label>
        <div class="analyse-draft-grid">
          <label><span>类型</span><select data-analyse-role="search-type"><option value="normal">普通</option><option value="escaped">扩展</option><option value="regex">正则</option><option value="regexMultiline">跨行正则</option></select></label>
          <label><span>选择</span><select data-analyse-role="selection"><option value="line">整行</option><option value="text">文本</option></select></label>
          <label><span>Order</span><input data-analyse-role="order" /></label>
          <label><span>Group</span><input data-analyse-role="group" /></label>
          <label><span>Comment</span><input data-analyse-role="comment" /></label>
          <label><span>Enter</span><select data-analyse-role="enter-action"><option value="update">更新当前行</option><option value="add">新增一行</option><option value="search">仅搜索</option></select></label>
          <label><span>前景色</span><input data-analyse-role="foreground" type="color" value="${DEFAULT_FOREGROUND}" /></label>
          <label><span>背景色</span><input data-analyse-role="background" type="color" value="${DEFAULT_BACKGROUND}" /></label>
        </div>
        <div class="analyse-flags">
          <label><input data-analyse-role="enabled" type="checkbox" checked />启用</label>
          <label><input data-analyse-role="match-case" type="checkbox" />区分大小写</label>
          <label><input data-analyse-role="whole-word" type="checkbox" />全词</label>
          <label><input data-analyse-role="hide" type="checkbox" />隐藏</label>
          <label><input data-analyse-role="bold" type="checkbox" />粗体</label>
          <label><input data-analyse-role="italic" type="checkbox" />斜体</label>
          <label><input data-analyse-role="underline" type="checkbox" />下划线</label>
        </div>
        <div class="analyse-actions">
          <button class="primary" data-analyse-role="add" type="button">添加</button>
          <button data-analyse-role="add-selection" type="button">添加源选区</button>
          <button data-analyse-role="update" type="button">更新</button>
          <button data-analyse-role="delete" type="button">删除</button>
          <button data-analyse-role="clear-patterns" type="button">清空 Pattern</button>
          <button data-analyse-role="up" type="button">上移</button>
          <button data-analyse-role="down" type="button">下移</button>
          <button data-analyse-role="sort" type="button">按 Order 排序</button>
          <button data-analyse-role="enable-all" type="button">全部启用</button>
          <button data-analyse-role="disable-all" type="button">全部禁用</button>
          <button data-analyse-role="enable-group" type="button">启用当前组</button>
          <button data-analyse-role="disable-group" type="button">禁用当前组</button>
        </div>
      </section>
      <section class="analyse-patterns">
        <table>
          <thead><tr><th>启用</th><th>Order</th><th>Search Text</th><th>类型</th><th>Group</th><th>Hits</th><th>状态</th></tr></thead>
          <tbody data-analyse-role="pattern-list"></tbody>
        </table>
      </section>
      <section class="analyse-result">
        <header>
          <div><strong>结果</strong><span data-analyse-role="result-summary">暂无结果</span></div>
          <div class="analyse-result-options">
            <label><input data-analyse-role="show-lines" type="checkbox" checked />源行号</label>
            <label><input data-analyse-role="word-wrap" type="checkbox" />自动换行</label>
            <label><input data-analyse-role="scroll-sync" type="checkbox" />滚动同步</label>
            <label>字号 <input data-analyse-role="result-font" class="analyse-font-size" type="number" min="10" max="24" value="12" /></label>
          </div>
        </header>
        <div class="analyse-result-find">
          <input data-analyse-role="result-find-input" aria-label="在 Analyse 结果中查找" placeholder="在结果中查找" />
          <select data-analyse-role="result-find-type" aria-label="结果查找类型"><option value="normal">普通</option><option value="escaped">扩展</option><option value="regex">正则</option><option value="regexMultiline">跨行正则</option></select>
          <label><input data-analyse-role="result-find-case" type="checkbox" />Aa</label>
          <label><input data-analyse-role="result-find-word" type="checkbox" />词</label>
          <button data-analyse-role="result-find-previous" type="button" title="上一个">↑</button>
          <button data-analyse-role="result-find-next" type="button" title="下一个">↓</button>
          <span data-analyse-role="result-find-summary"></span>
        </div>
        <div class="analyse-result-actions">
          <button data-analyse-role="copy-result" type="button">复制</button>
          <button data-analyse-role="copy-result-rtf" type="button">复制富文本</button>
          <button data-analyse-role="save-result" type="button">保存文本</button>
          <button data-analyse-role="save-result-rtf" type="button">保存 RTF</button>
          <button data-analyse-role="bind-result" type="button">绑定文件</button>
          <button data-analyse-role="unbind-result" type="button">解绑</button>
          <span data-analyse-role="bound-result">未绑定结果文件</span>
        </div>
        <div class="analyse-result-editor" data-analyse-role="result-editor"></div>
        <div class="analyse-matching-patterns" data-analyse-role="matching-patterns">当前行无匹配 Pattern</div>
      </section>
      <footer class="analyse-status" data-analyse-role="status" aria-live="polite"></footer>
    </div>
  `;
}
