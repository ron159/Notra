export interface BatchResult<T> {
  value?: T;
  error?: unknown;
}

export interface BatchProgress<T> {
  completed: number;
  index: number;
  result: BatchResult<T>;
  total: number;
}

export interface BatchOptions<T> {
  concurrency?: number;
  priorityIndex?: number;
  onProgress?: (progress: BatchProgress<T>) => void;
}

export async function mapWithConcurrency<TInput, TOutput>(
  items: readonly TInput[],
  worker: (item: TInput, index: number) => Promise<TOutput>,
  options: BatchOptions<TOutput> = {},
): Promise<BatchResult<TOutput>[]> {
  const results = new Array<BatchResult<TOutput>>(items.length);
  if (items.length === 0) return results;

  const priorityIndex = normalizePriorityIndex(options.priorityIndex, items.length);
  const pending = Array.from({ length: items.length }, (_, index) => index);
  if (priorityIndex > 0) {
    pending.splice(priorityIndex, 1);
    pending.unshift(priorityIndex);
  }

  const concurrency = Math.min(
    pending.length,
    Math.max(1, Math.floor(options.concurrency ?? 4)),
  );
  let cursor = 0;
  let completed = 0;

  const runWorker = async () => {
    while (cursor < pending.length) {
      const pendingIndex = cursor++;
      const itemIndex = pending[pendingIndex];
      let result: BatchResult<TOutput>;
      try {
        result = { value: await worker(items[itemIndex], itemIndex) };
      } catch (error) {
        result = { error };
      }
      results[itemIndex] = result;
      completed += 1;
      options.onProgress?.({ completed, index: itemIndex, result, total: items.length });
    }
  };

  await Promise.all(Array.from({ length: concurrency }, runWorker));
  return results;
}

function normalizePriorityIndex(index: number | undefined, length: number) {
  if (!Number.isInteger(index) || index === undefined || index < 0 || index >= length) return 0;
  return index;
}
