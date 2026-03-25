$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "rust\Cargo.toml"
$includeHeader = Join-Path $root "include\p4_core.h"

Push-Location $root
try {
    cargo build --manifest-path $manifest --release -p p4-ffi

    $targetDir = Join-Path $root "rust\target\release"
    $outDir = Join-Path $root "dist\p4_core\windows-x64"
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null

    Copy-Item $includeHeader (Join-Path $outDir "p4_core.h") -Force
    Copy-Item (Join-Path $targetDir "p4_ffi.dll") (Join-Path $outDir "p4_core.dll") -Force
    if (Test-Path (Join-Path $targetDir "p4_ffi.lib")) {
        Copy-Item (Join-Path $targetDir "p4_ffi.lib") (Join-Path $outDir "p4_core.lib") -Force
    }
    if (Test-Path (Join-Path $targetDir "p4_ffi.dll.lib")) {
        Copy-Item (Join-Path $targetDir "p4_ffi.dll.lib") (Join-Path $outDir "p4_core.dll.lib") -Force
    }
    Copy-Item (Join-Path $targetDir "p4_ffi.a") (Join-Path $outDir "libp4_core.a") -Force -ErrorAction SilentlyContinue

    Write-Host "Built P4 core artifacts in $outDir"
}
finally {
    Pop-Location
}



