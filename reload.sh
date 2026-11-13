#!/usr/bin/env bash
#
# Development and Live Testing Reload Script
# Pushes the latest binaries, scripts, configuration, and WebUI assets directly
# to the target Android device via ADB without requiring a full device reboot.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_ID="oplus_hardware_hub"
REMOTE_MOD_DIR="/data/adb/modules/${MODULE_ID}"

TARGET_SERIAL="${TARGET_SERIAL:-10.0.0.12:5555}"
if [ "${1:-}" != "" ]; then
    TARGET_SERIAL="$1"
fi

echo "[*] Target device: ${TARGET_SERIAL}"

echo "[*] Triggering compilation via build.sh..."
bash "${SCRIPT_DIR}/build.sh"

echo "[*] Ensuring remote module directory exists on target..."
adb -s "${TARGET_SERIAL}" shell "su -c 'mkdir -p ${REMOTE_MOD_DIR}/bin ${REMOTE_MOD_DIR}/webroot'"

echo "[*] Stopping running daemons on target..."
adb -s "${TARGET_SERIAL}" shell "su -c 'pkill -9 -x ohh || true'"

echo "[*] Pushing module files..."
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/module.prop" "/data/local/tmp/module.prop"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/config.conf" "/data/local/tmp/config.conf"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/service.sh" "/data/local/tmp/service.sh"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/post-fs-data.sh" "/data/local/tmp/post-fs-data.sh"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/customize.sh" "/data/local/tmp/customize.sh"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/uninstall.sh" "/data/local/tmp/uninstall.sh"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/bin/ohh" "/data/local/tmp/ohh"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/webroot" "/data/local/tmp/"

adb -s "${TARGET_SERIAL}" shell "su -c '
    cp /data/local/tmp/module.prop ${REMOTE_MOD_DIR}/module.prop
    cp /data/local/tmp/config.conf ${REMOTE_MOD_DIR}/config.conf
    cp /data/local/tmp/service.sh ${REMOTE_MOD_DIR}/service.sh
    cp /data/local/tmp/post-fs-data.sh ${REMOTE_MOD_DIR}/post-fs-data.sh
    cp /data/local/tmp/customize.sh ${REMOTE_MOD_DIR}/customize.sh
    cp /data/local/tmp/uninstall.sh ${REMOTE_MOD_DIR}/uninstall.sh
    cp /data/local/tmp/ohh ${REMOTE_MOD_DIR}/bin/ohh
    rm -rf ${REMOTE_MOD_DIR}/webroot
    cp -r /data/local/tmp/webroot ${REMOTE_MOD_DIR}/
    chmod 0755 ${REMOTE_MOD_DIR}/service.sh ${REMOTE_MOD_DIR}/post-fs-data.sh ${REMOTE_MOD_DIR}/customize.sh ${REMOTE_MOD_DIR}/uninstall.sh ${REMOTE_MOD_DIR}/bin/ohh
    chmod 0644 ${REMOTE_MOD_DIR}/module.prop ${REMOTE_MOD_DIR}/config.conf
    rm -rf /data/local/tmp/module.prop /data/local/tmp/config.conf /data/local/tmp/service.sh /data/local/tmp/post-fs-data.sh /data/local/tmp/customize.sh /data/local/tmp/uninstall.sh /data/local/tmp/ohh /data/local/tmp/webroot
'"

echo "[*] Starting supervisor in background on target..."
adb -s "${TARGET_SERIAL}" shell "su -c 'nohup /data/adb/modules/${MODULE_ID}/service.sh >/dev/null 2>&1 &'"

sleep 2

echo "[*] Verifying daemon status..."
adb -s "${TARGET_SERIAL}" shell "su -c '
    echo \"PID: \$(pidof ohh || echo not_running)\"
    ${REMOTE_MOD_DIR}/bin/ohh --status
'"

echo "[+] Reload completed successfully on ${TARGET_SERIAL}."
