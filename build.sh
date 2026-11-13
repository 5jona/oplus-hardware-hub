#!/usr/bin/env bash
#
# OnePlus Smart FastCharge & Thermal Governor - Packaging Script
# Compiles the aarch64 Android daemon and packages module components into a flashable ZIP.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GOVERNOR_DIR="${SCRIPT_DIR}/governor"
DAEMON_TARGET_TRIPLE="aarch64-linux-android"
DAEMON_BIN_SRC="${GOVERNOR_DIR}/target/${DAEMON_TARGET_TRIPLE}/release/charge_governor"
DAEMON_BIN_DST="${SCRIPT_DIR}/bin/charge_governor"
OUTPUT_DIR="${SCRIPT_DIR}/out"
PACKAGE_NAME="oplus-smart-charge.zip"

echo "[*] Compiling charge_governor (${DAEMON_TARGET_TRIPLE})..."
(
    cd "${GOVERNOR_DIR}"
    cargo build --release --target "${DAEMON_TARGET_TRIPLE}"
)

if [ ! -f "${DAEMON_BIN_SRC}" ]; then
    echo "[!] Error: Compiled daemon binary not found at ${DAEMON_BIN_SRC}" >&2
    exit 1
fi

mkdir -p "${SCRIPT_DIR}/bin" "${OUTPUT_DIR}"
cp "${DAEMON_BIN_SRC}" "${DAEMON_BIN_DST}"
chmod 0755 "${DAEMON_BIN_DST}"

echo "[*] Packaging KernelSU module..."
rm -f "${OUTPUT_DIR}/${PACKAGE_NAME}"

python3 - <<EOF
import os
import stat
import zipfile

script_dir = "${SCRIPT_DIR}"
output_zip = os.path.join("${OUTPUT_DIR}", "${PACKAGE_NAME}")

files_to_pack = [
    ("module.prop", 0o644),
    ("config.conf", 0o644),
    ("service.sh", 0o755),
    ("post-fs-data.sh", 0o755),
    ("customize.sh", 0o755),
    ("uninstall.sh", 0o755),
    ("bin/charge_governor", 0o755),
]

def write_with_mode(zf, src_path, arcname, mode):
    with open(src_path, "rb") as f:
        data = f.read()
    info = zipfile.ZipInfo(arcname)
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = (stat.S_IFREG | mode) << 16
    zf.writestr(info, data)

with zipfile.ZipFile(output_zip, "w", zipfile.ZIP_DEFLATED) as zf:
    for filename, mode in files_to_pack:
        file_path = os.path.join(script_dir, filename)
        if os.path.isfile(file_path):
            write_with_mode(zf, file_path, filename, mode)

    for folder in ("webroot",):
        target_dir = os.path.join(script_dir, folder)
        if not os.path.isdir(target_dir):
            continue
        for root, dirs, files in os.walk(target_dir):
            dirs[:] = [d for d in dirs if not d.startswith(".")]
            for f in files:
                if f.startswith(".") or f.endswith((".tmp", ".swp", "~")):
                    continue
                full_path = os.path.join(root, f)
                arcname = os.path.relpath(full_path, script_dir)
                write_with_mode(zf, full_path, arcname, 0o644)

print("[+] KernelSU module archive created:")
print(f"    {output_zip}")
EOF
