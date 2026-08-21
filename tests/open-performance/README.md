# Multi-file open performance

This benchmark covers the path from a multi-file request to an interactive active document and the completed background restore.

Generate deterministic fixtures outside the repository:

```bash
cd crates/otterdive-app/frontend
npm run benchmark:fixtures -- --count 50 --size-kb 256 --kind text
npm run benchmark:fixtures -- --count 50 --size-kb 256 --kind markdown
```

Run text and Markdown cases for this matrix:

| File count | Size per file |
| ---: | ---: |
| 10, 50, 100 | 4 KiB |
| 10, 50, 100 | 256 KiB |
| 10, 50, 100 | 2 MiB |

Drag every file from one generated directory into OtterDive at once, or pass them as startup arguments. The application log records:

- `会话活动文件可用`: startup-to-interactive time for the restored active file.
- `批量打开性能` and `会话已恢复`: total duration, Long Task count, and Long Task duration.
- JavaScript heap delta when Chromium exposes `performance.memory`; WebKit reports it as unavailable.

For a regression comparison, use the same machine and build mode. At minimum, compare the 10/50/100-file 256 KiB text cases and run:

```bash
cd crates/otterdive-app/frontend
npm run test:performance
npm run build
```

The automated tests enforce bounded concurrency, stable result order, failure isolation, and the absence of per-file activation in multi-file entry points.
