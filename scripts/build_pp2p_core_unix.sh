#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${ROOT_DIR}/rust/target/release"
OUT_BASE="${ROOT_DIR}/dist/pp2p_core"
HEADER="${ROOT_DIR}/include/pp2p_core.h"

cd "${ROOT_DIR}"
cargo build --manifest-path "${ROOT_DIR}/rust/Cargo.toml" --release -p pp2p-ffi

platform_dir="linux-x64"
lib_name_src="libpp2p_ffi.so"
lib_name_dst="libpp2p_core.so"

case "$(uname -s)" in
  Darwin)
    platform_dir="macos"
    lib_name_src="libpp2p_ffi.dylib"
    lib_name_dst="libpp2p_core.dylib"
    ;;
  Linux)
    platform_dir="linux-x64"
    lib_name_src="libpp2p_ffi.so"
    lib_name_dst="libpp2p_core.so"
    ;;
esac

OUT_DIR="${OUT_BASE}/${platform_dir}"
mkdir -p "${OUT_DIR}"

cp -f "${HEADER}" "${OUT_DIR}/pp2p_core.h"
cp -f "${TARGET_DIR}/${lib_name_src}" "${OUT_DIR}/${lib_name_dst}"
cp -f "${TARGET_DIR}/libpp2p_ffi.a" "${OUT_DIR}/libpp2p_core.a" || true

echo "Built PP2P core artifacts in ${OUT_DIR}"

