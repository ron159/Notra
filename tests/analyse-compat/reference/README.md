# Windows Oracle Preparation

This directory contains only an independently authored manifest and preparation
script. It does not redistribute AnalysePlugin, Notepad++, their source, or the
AnalysePlugin XSD.

## Prepare the frozen runtime

Run on a clean Windows x64 host from PowerShell:

```powershell
.\prepare-oracle.ps1 -OutputDirectory C:\otterdive-analyse-oracle
```

The script:

1. downloads the official AnalysePlugin r54 and Notepad++ 8.4.8 x64 archives;
2. rejects either archive if its SHA-256 differs from the manifest;
3. verifies the extracted Notepad++ executable and AnalysePlugin x64 DLL;
4. installs the DLL into the portable Notepad++ plugin layout;
5. records Windows version/build/architecture and artifact hashes in
   `runtime\oracle-environment.json`.

It refuses to reuse an existing `runtime` directory so stale binaries cannot be
silently mixed into a new Oracle run. Downloads may be reused only when their
hashes still match.

## Collect a Golden case

Use only checked-in files under `fixtures/` and `profiles/`; never use private
documents or user logs.

For each case:

1. launch `runtime\NotepadPlusPlus\notepad++.exe`;
2. confirm AnalysePlugin reports release 1.14 revision 54;
3. open the fixture and load the matching XML profile in Replace mode;
4. run Analyse and record source lines, total matches, matching Pattern order,
   Pattern errors, and resolved style/hide ranges;
5. save semantic output under `expected/` and remove the
   `provisional_not_binary_verified` marker only when the values came from this
   frozen runtime;
6. preserve `oracle-environment.json` with the run evidence.

Screenshots may supplement the structured output but cannot replace it. Regex,
Whole Word, Unicode case folding, zero-length matching, and Extended surrogate
cases remain blocked until they have recorded output from this runtime.
