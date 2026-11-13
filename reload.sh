#!/usr/bin/env bash
#
# Development and Live Testing Reload Script
# Pushes the latest binaries, scripts, configuration, and WebUI assets directly
# to the target Android device via ADB without requiring a full device reboot.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_ID="oplus_smart_charge"
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

echo "[*] Stopping running daemon on target..."
adb -s "${TARGET_SERIAL}" shell "su -c 'pkill -9 -x charge_governor || true'"

echo "[*] Pushing module files..."
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/module.prop" "/data/local/tmp/module.prop"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/config.conf" "/data/local/tmp/config.conf"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/service.sh" "/data/local/tmp/service.sh"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/bin/charge_governor" "/data/local/tmp/charge_governor"
adb -s "${TARGET_SERIAL}" push "${SCRIPT_DIR}/webroot" "/data/local/tmp/"

adb -s "${TARGET_SERIAL}" shell "su -c '
    cp /data/local/tmp/module.prop ${REMOTE_MOD_DIR}/module.prop
    [ -f ${REMOTE_MOD_DIR}/config.conf ] || cp /data/local/tmp/config.conf ${REMOTE_MOD_DIR}/config.conf
    cp /data/local/tmp/service.sh ${REMOTE_MOD_DIR}/service.sh
    cp /data/local/tmp/charge_governor ${REMOTE_MOD_DIR}/bin/charge_governor
    rm -rf ${REMOTE_MOD_DIR}/webroot
    cp -r /data/local/tmp/webroot ${REMOTE_MOD_DIR}/
    chmod 0755 ${REMOTE_MOD_DIR}/service.sh ${REMOTE_MOD_DIR}/bin/charge_governor
    chmod 0644 ${REMOTE_MOD_DIR}/module.prop ${REMOTE_MOD_DIR}/config.conf
    rm -rf /data/local/tmp/module.prop /data/local/tmp/config.conf /data/local/tmp/service.sh /data/local/tmp/charge_governor /data/local/tmp/webroot
'"

echo "[*] Starting supervisor in background on target..."
adb -s "${TARGET_SERIAL}" shell "su -c 'nohup /data/adb/modules/${MODULE_ID}/service.sh >/dev/null 2>&1 &'"

sleep 2

echo "[*] Verifying daemon status..."
adb -s "${TARGET_SERIAL}" shell "su -c '
    echo \"PID: \$(pidof charge_governor || echo not_running)\"
    ${REMOTE_MOD_DIR}/bin/charge_governor --status
'"

echo "[+] Reload completed successfully on ${TARGET_SERIAL}."
