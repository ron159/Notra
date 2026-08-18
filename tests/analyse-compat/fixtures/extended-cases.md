# Phase 1 Extended Cases

The executable corpus lives in `otterdive-core::analyse::extended` unit tests. It
covers simple escapes, binary/octal/decimal/hex values, `\uHHHH`, mixed Unicode,
invalid and truncated fallback, NUL, trailing backslash, and UTF-16 surrogate
pairs.

Lone surrogate behavior is intentionally reported as an error because Rust
strings cannot contain an unpaired UTF-16 code unit. This case remains subject to
the fixed binary Oracle gate.
