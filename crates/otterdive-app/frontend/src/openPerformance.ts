export interface OpenPerformanceMark {
  heapBytes: number | null;
  longTaskCount: number;
  longTaskDurationMs: number;
  startedAt: number;
  total: number;
}

export interface OpenPerformanceSummary {
  durationMs: number;
  failed: number;
  heapDeltaBytes: number | null;
  longTaskCount: number;
  longTaskDurationMs: number;
  opened: number;
  total: number;
}

let observedLongTaskCount = 0;
let observedLongTaskDurationMs = 0;

if (typeof PerformanceObserver !== "undefined") {
  try {
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        observedLongTaskCount += 1;
        observedLongTaskDurationMs += entry.duration;
      }
    });
    observer.observe({ type: "longtask", buffered: true });
  } catch {
    // WebKit does not expose the Long Tasks API on every supported platform.
  }
}

export function startOpenPerformance(total: number): OpenPerformanceMark {
  return {
    heapBytes: usedHeapBytes(),
    longTaskCount: observedLongTaskCount,
    longTaskDurationMs: observedLongTaskDurationMs,
    startedAt: performance.now(),
    total,
  };
}

export function finishOpenPerformance(
  mark: OpenPerformanceMark,
  opened: number,
  failed: number,
): OpenPerformanceSummary {
  return summarizeOpenPerformance({
    durationMs: performance.now() - mark.startedAt,
    failed,
    heapDeltaBytes: heapDelta(mark.heapBytes, usedHeapBytes()),
    longTaskCount: observedLongTaskCount - mark.longTaskCount,
    longTaskDurationMs: observedLongTaskDurationMs - mark.longTaskDurationMs,
    opened,
    total: mark.total,
  });
}

export function summarizeOpenPerformance(summary: OpenPerformanceSummary): OpenPerformanceSummary {
  return {
    ...summary,
    durationMs: Math.max(0, summary.durationMs),
    failed: Math.max(0, Math.floor(summary.failed)),
    longTaskCount: Math.max(0, Math.floor(summary.longTaskCount)),
    longTaskDurationMs: Math.max(0, summary.longTaskDurationMs),
    opened: Math.max(0, Math.floor(summary.opened)),
    total: Math.max(0, Math.floor(summary.total)),
  };
}

function heapDelta(start: number | null, end: number | null) {
  return start === null || end === null ? null : end - start;
}

function usedHeapBytes() {
  const memory = (performance as Performance & {
    memory?: { usedJSHeapSize?: number };
  }).memory;
  return typeof memory?.usedJSHeapSize === "number" ? memory.usedJSHeapSize : null;
}
