# Analyse large-file benchmark

This directory records reproducible performance evidence for the Analyse large-file route. It is not a binary-compatibility oracle.

Build the benchmark in release mode:

```sh
cargo build -p otterdive-core --release --example analyse_large_file_benchmark
```

Run both input modes for a supported size:

```sh
target/release/examples/analyse_large_file_benchmark --size-mb 100 --mode owned
target/release/examples/analyse_large_file_benchmark --size-mb 100 --mode mmap
```

Supported generated fixture sizes are 1–1024 MiB. The fixture is deterministic ASCII text with one `ERROR code=9001` record per MiB. Each run uses the same Normal and lookbehind Regex patterns, and prints one JSON result line.

`mmap` is currently benchmark-only. The generated fixture is owned by the benchmark and cannot change while mapped. Production documents can be modified or truncated by another process, so OtterDive keeps the safe owned read path until it has an immutable snapshot or a cross-platform file-stability contract.
