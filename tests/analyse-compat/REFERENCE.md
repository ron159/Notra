# Analyse Compatibility Reference

This directory stores independently authored inputs and expected behavior for the
Notra Analyse compatibility harness. It must not contain AnalysePlugin source,
its XSD, or other GPL implementation files.

## Frozen reference

| Item | Current value |
|---|---|
| AnalysePlugin source | SourceForge SVN r54 snapshot |
| Source snapshot path | External to this repository |
| AnalysePlugin release | `AnalysePlugin-v01.14-R54-ALL.zip` |
| Release archive SHA-256 | `d231d9ade39c60384db9063bc265ba0eb210178e6c4648275ee0e07a3ea943c0` |
| AnalysePlugin x64 DLL SHA-256 | `502ce25f7553cc05928df8e3c6b56827bb2eee9db82a94edcb71ef1496c71594` |
| AnalysePlugin x86 DLL SHA-256 | `9b0b399991e0c80be8cd1b01d1d2b1227ccee5210a03216d20165eddc534737e` |
| Notepad++ release | 8.4.8 portable x64 |
| Notepad++ archive SHA-256 | `29eba8e7760db4b30d9d664b3de66b33c855036c579a306ede73c651453a4409` |
| `notepad++.exe` SHA-256 | `6eebed1fd47637616e93a797fe061d6504ad81454a822ec3bfd172a0f922c884` |
| Notepad++ architecture | x86-64 |
| Windows version | Not yet frozen |
| Notra Regex backend | `fancy-regex` 0.14.0, MIT |
| Reference Regex runtime | Not yet identified from executed Golden cases |

Official downloads:

- `https://sourceforge.net/projects/analyseplugin/files/binaries/v01.14-R54/AnalysePlugin-v01.14-R54-ALL.zip/download`
- `https://github.com/notepad-plus-plus/notepad-plus-plus/releases/download/v8.4.8/npp.8.4.8.portable.x64.zip`

The release archives and PE architecture have been verified. This macOS host has
no Wine, QEMU, VirtualBox, or Parallels command-line runtime, so Windows execution
and Golden output generation remain open. These cases are Release compatibility
tests; they do not block functional implementation or UI development.

Notra uses one pure-Rust Regex backend on all supported platforms. The Analyse
adapter adds `\R`, line-anchor defaults, Dot Matches Newline selection, a
backtrack limit, zero-length progress, Pattern-level errors, and cancellation
checks. The Matcher trait remains replaceable if Release testing finds a
function-blocking difference.

`reference/oracle-manifest.json` is the machine-readable source of artifact
URLs and hashes. `reference/prepare-oracle.ps1` verifies those values, installs
the x64 plugin into portable Notepad++, and records Windows environment evidence.

## Golden generation

1. Run the fixture text and profile in the frozen Windows reference environment.
2. Record semantic output under `expected/`: source lines, match counts, matching
   pattern order, resolved style segments, and errors.
3. Run the same input through `notra-core`.
4. Compare structured output. Screenshots are supplementary evidence only.

Phase 1 fixtures cover domain defaults, extended translation, and XML profile
semantics. Phase 2 adds a structured Normal-search/Merge/Style contract. Any
expected file not produced by the binary Oracle carries an explicit
`provisional_not_binary_verified` marker. Binary-dependent results remain
provisional compatibility evidence, while deterministic Notra contracts are
valid functional regression tests.
