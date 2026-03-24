$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "rust\Cargo.toml"
$includeHeader = Join-Path $root "include\pp2p_core.h"

Push-Location $root
try {
    cargo build --manifest-path $manifest --release -p pp2p-ffi

    $targetDir = Join-Path $root "rust\target\release"
    $outDir = Join-Path $root "dist\pp2p_core\windows-x64"
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null

    Copy-Item $includeHeader (Join-Path $outDir "pp2p_core.h") -Force
    Copy-Item (Join-Path $targetDir "pp2p_ffi.dll") (Join-Path $outDir "pp2p_core.dll") -Force
    if (Test-Path (Join-Path $targetDir "pp2p_ffi.lib")) {
        Copy-Item (Join-Path $targetDir "pp2p_ffi.lib") (Join-Path $outDir "pp2p_core.lib") -Force
    }
    if (Test-Path (Join-Path $targetDir "pp2p_ffi.dll.lib")) {
        Copy-Item (Join-Path $targetDir "pp2p_ffi.dll.lib") (Join-Path $outDir "pp2p_core.dll.lib") -Force
    }
    Copy-Item (Join-Path $targetDir "pp2p_ffi.a") (Join-Path $outDir "libpp2p_core.a") -Force -ErrorAction SilentlyContinue

    Write-Host "Built PP2P core artifacts in $outDir"
}
finally {
    Pop-Location
}

