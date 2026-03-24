#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_DIR="${ROOT_DIR}/tor_win_min_src"
OUT_DIR="${1:-${ROOT_DIR}/dist}"
OUT_NAME="${2:-onionrelay}"

if [[ ! -d "${SRC_DIR}" ]]; then
  echo "Missing source directory: ${SRC_DIR}" >&2
  exit 1
fi

if command -v getconf >/dev/null 2>&1; then
  JOBS="$(getconf _NPROCESSORS_ONLN || echo 4)"
else
  JOBS=4
fi

CONFIG_FLAGS=(
  --disable-asciidoc
  --disable-module-relay
  --disable-module-dirauth
  --disable-module-pow
)

if [[ "$(uname -s)" == "Darwin" ]]; then
  OPENSSL_PREFIX="$(brew --prefix openssl@3 2>/dev/null || true)"
  LIBEVENT_PREFIX="$(brew --prefix libevent 2>/dev/null || true)"
  XZ_PREFIX="$(brew --prefix xz 2>/dev/null || true)"
  ZSTD_PREFIX="$(brew --prefix zstd 2>/dev/null || true)"

  for prefix in "$OPENSSL_PREFIX" "$LIBEVENT_PREFIX" "$XZ_PREFIX" "$ZSTD_PREFIX"; do
    if [[ -n "${prefix}" ]]; then
      export PKG_CONFIG_PATH="${prefix}/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
      export CPPFLAGS="-I${prefix}/include ${CPPFLAGS:-}"
      export LDFLAGS="-L${prefix}/lib ${LDFLAGS:-}"
    fi
  done
fi

pushd "${SRC_DIR}" >/dev/null

# Clean stale dependency metadata that can be copied from Windows/MSYS builds.
# These files can make GNU make fail on Linux with "multiple target patterns".
find . -type d -name .deps -prune -exec rm -rf {} + || true
find . -name "*.Po" -delete || true
find . -name "*.Plo" -delete || true
rm -f config.log config.status || true

make distclean >/dev/null 2>&1 || true
./configure "${CONFIG_FLAGS[@]}"
make -j"${JOBS}" src/app/tor libtor.a
popd >/dev/null

mkdir -p "${OUT_DIR}"
cp -f "${SRC_DIR}/src/app/tor" "${OUT_DIR}/${OUT_NAME}"
chmod +x "${OUT_DIR}/${OUT_NAME}"

echo "Built ${OUT_DIR}/${OUT_NAME}"
