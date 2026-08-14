[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory
)

$ErrorActionPreference = "Stop"

function Assert-Sha256 {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$Expected
    )

    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $Expected.ToLowerInvariant()) {
        throw "SHA-256 mismatch for '$Path'. Expected $Expected, got $actual."
    }
}

function Get-VerifiedArtifact {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Uri,
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$Sha256
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $Path
    }
    Assert-Sha256 -Path $Path -Expected $Sha256
}

if (-not $IsWindows -and $PSVersionTable.PSEdition -eq "Core") {
    throw "The Analyse reference Oracle must be prepared on Windows."
}

$manifestPath = Join-Path $PSScriptRoot "oracle-manifest.json"
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$outputRoot = [System.IO.Path]::GetFullPath($OutputDirectory)
$downloadDirectory = Join-Path $outputRoot "downloads"
$runtimeDirectory = Join-Path $outputRoot "runtime"

if (Test-Path -LiteralPath $runtimeDirectory) {
    throw "Runtime directory already exists: '$runtimeDirectory'. Use a new output directory."
}

New-Item -ItemType Directory -Path $downloadDirectory -Force | Out-Null
$pluginArchive = Join-Path $downloadDirectory $manifest.analysePlugin.archiveFile
$notepadArchive = Join-Path $downloadDirectory $manifest.notepadPlusPlus.archiveFile

Get-VerifiedArtifact `
    -Uri $manifest.analysePlugin.archiveUrl `
    -Path $pluginArchive `
    -Sha256 $manifest.analysePlugin.archiveSha256
Get-VerifiedArtifact `
    -Uri $manifest.notepadPlusPlus.archiveUrl `
    -Path $notepadArchive `
    -Sha256 $manifest.notepadPlusPlus.archiveSha256

$notepadDirectory = Join-Path $runtimeDirectory "NotepadPlusPlus"
$pluginExtractDirectory = Join-Path $runtimeDirectory "AnalysePluginArchive"
New-Item -ItemType Directory -Path $notepadDirectory -Force | Out-Null
New-Item -ItemType Directory -Path $pluginExtractDirectory -Force | Out-Null
Expand-Archive -LiteralPath $notepadArchive -DestinationPath $notepadDirectory
Expand-Archive -LiteralPath $pluginArchive -DestinationPath $pluginExtractDirectory

$notepadExecutable = Join-Path $notepadDirectory $manifest.notepadPlusPlus.executable
Assert-Sha256 -Path $notepadExecutable -Expected $manifest.notepadPlusPlus.executableSha256

$sourceDll = Join-Path $pluginExtractDirectory $manifest.analysePlugin.x64DllMember
Assert-Sha256 -Path $sourceDll -Expected $manifest.analysePlugin.x64DllSha256
$pluginDirectory = Join-Path (Join-Path $notepadDirectory "plugins") "AnalysePlugin"
New-Item -ItemType Directory -Path $pluginDirectory -Force | Out-Null
$installedDll = Join-Path $pluginDirectory "AnalysePlugin.dll"
Copy-Item -LiteralPath $sourceDll -Destination $installedDll
Assert-Sha256 -Path $installedDll -Expected $manifest.analysePlugin.x64DllSha256

$operatingSystem = Get-CimInstance Win32_OperatingSystem
$evidence = [ordered]@{
    schemaVersion = 1
    generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    windows = [ordered]@{
        caption = $operatingSystem.Caption
        version = $operatingSystem.Version
        buildNumber = $operatingSystem.BuildNumber
        architecture = $operatingSystem.OSArchitecture
    }
    analysePlugin = [ordered]@{
        release = $manifest.analysePlugin.release
        x64DllSha256 = $manifest.analysePlugin.x64DllSha256
    }
    notepadPlusPlus = [ordered]@{
        release = $manifest.notepadPlusPlus.release
        architecture = $manifest.notepadPlusPlus.architecture
        archiveSha256 = $manifest.notepadPlusPlus.archiveSha256
        executableSha256 = $manifest.notepadPlusPlus.executableSha256
    }
}
$evidencePath = Join-Path $runtimeDirectory "oracle-environment.json"
$evidence | ConvertTo-Json -Depth 5 | Out-File -LiteralPath $evidencePath -Encoding utf8

Write-Host "Analyse Oracle prepared successfully."
Write-Host "Executable: $notepadExecutable"
Write-Host "Environment evidence: $evidencePath"
