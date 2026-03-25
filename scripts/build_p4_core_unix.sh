#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${ROOT_DIR}/rust/target/release"
OUT_BASE="${ROOT_DIR}/dist/p4_core"
HEADER="${ROOT_DIR}/include/p4_core.h"

cd "${ROOT_DIR}"
cargo build --manifest-path "${ROOT_DIR}/rust/Cargo.toml" --release -p p4-ffi

platform_dir="linux-x64"
lib_name_src="libp4_ffi.so"
lib_name_dst="libp4_core.so"

case "$(uname -s)" in
  Darwin)
    platform_dir="macos"
    lib_name_src="libp4_ffi.dylib"
    lib_name_dst="libp4_core.dylib"
    ;;
  Linux)
    platform_dir="linux-x64"
    lib_name_src="libp4_ffi.so"
    lib_name_dst="libp4_core.so"
    ;;
esac

OUT_DIR="${OUT_BASE}/${platform_dir}"
mkdir -p "${OUT_DIR}"

cp -f "${HEADER}" "${OUT_DIR}/p4_core.h"
cp -f "${TARGET_DIR}/${lib_name_src}" "${OUT_DIR}/${lib_name_dst}"
cp -f "${TARGET_DIR}/libp4_ffi.a" "${OUT_DIR}/libp4_core.a" || true

echo "Built P4 core artifacts in ${OUT_DIR}"



