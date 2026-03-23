param(
    [string]$SourceDir = "torbackup/tor",
    [string]$OutputDir = "tor_win_min_src"
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path ".").Path
$bash = "C:\msys64\usr\bin\bash.exe"
if (-not (Test-Path $bash)) {
    throw "MSYS2 bash not found at $bash"
}

$srcFull = (Resolve-Path $SourceDir).Path
$outFull = Join-Path $repoRoot $OutputDir
$cfgLog = Join-Path $repoRoot "build\tor_win_min_configure.log"
$makeLog = Join-Path $repoRoot "build\tor_win_min_make.log"

function Convert-ToMsysPath([string]$windowsPath) {
    $resolved = (Resolve-Path $windowsPath).Path
    $drive = $resolved.Substring(0, 1).ToLowerInvariant()
    $tail = $resolved.Substring(2).Replace('\', '/')
    return "/$drive$tail"
}

$srcMsys = Convert-ToMsysPath $srcFull
$outParent = Convert-ToMsysPath $repoRoot
$cfgLogParent = Convert-ToMsysPath (Join-Path $repoRoot "build")

$bashScript = @'
source /etc/profile
set -euo pipefail
export MSYSTEM=MINGW64
export PATH=/mingw64/bin:/usr/bin:\$PATH

SRC_IN='__SRC_IN__'
SRC_OUT='__SRC_OUT__'
CFG_LOG='__CFG_LOG__'
MAKE_LOG='__MAKE_LOG__'

rm -rf "\$SRC_OUT"
cp -a "\$SRC_IN" "\$SRC_OUT"
cd "\$SRC_OUT"

make distclean >/dev/null 2>&1 || true

./configure \
  --disable-asciidoc \
  --disable-module-relay \
  --disable-module-dirauth \
  --disable-module-pow \
  > "\$CFG_LOG" 2>&1

make -j"\$(nproc)" src/app/tor.exe libtor.a > "\$MAKE_LOG" 2>&1

# Copy runtime DLL dependencies beside tor.exe for portable Windows runs.
cp -f /mingw64/bin/libcrypto-3-x64.dll src/app/
cp -f /mingw64/bin/libssl-3-x64.dll src/app/
cp -f /mingw64/bin/libevent-7.dll src/app/
cp -f /mingw64/bin/liblzma-5.dll src/app/
cp -f /mingw64/bin/zlib1.dll src/app/
cp -f /mingw64/bin/libzstd.dll src/app/
cp -f /mingw64/bin/libwinpthread-1.dll src/app/

echo "DONE"
'@

$bashScript = $bashScript.Replace("__SRC_IN__", $srcMsys)
$bashScript = $bashScript.Replace("__SRC_OUT__", "$outParent/$OutputDir")
$bashScript = $bashScript.Replace("__CFG_LOG__", "$cfgLogParent/tor_win_min_configure.log")
$bashScript = $bashScript.Replace("__MAKE_LOG__", "$cfgLogParent/tor_win_min_make.log")

& $bash -lc $bashScript

Write-Host ""
Write-Host "Build complete:"
Write-Host "  tor.exe : $outFull\src\app\tor.exe"
Write-Host "  libtor.a: $outFull\libtor.a"
Write-Host "  configure log: $cfgLog"
Write-Host "  make log     : $makeLog"
