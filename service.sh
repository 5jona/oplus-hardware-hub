#!/system/bin/sh
#
# Late-start service executed by KernelSU after boot completion.
# Supervises the charge_governor daemon lifecycle.
#

MODDIR="${0%/*}"
LOG_TAG="OplusSmartCharge"
DAEMON_BIN="$MODDIR/bin/charge_governor"
CONFIG_FILE="$MODDIR/config.conf"

log_info() {
    log -t "$LOG_TAG" -p i "$1"
}

# Wait until Android framework boot has fully completed
until [ "$(getprop sys.boot_completed)" = "1" ]; do
    sleep 2
done

log_info "Boot completed. Initializing OnePlus Smart FastCharge Governor..."

# Terminate any stale instances
pkill -9 -x "charge_governor" 2>/dev/null || true

if [ -x "$DAEMON_BIN" ]; then
    log_info "Starting charge_governor supervisor (config: $CONFIG_FILE)..."
    while true; do
        if [ -f "$MODDIR/remove" ]; then
            log_info "Module marked for removal. Exiting supervisor."
            break
        fi

        if [ -f "$MODDIR/disable" ]; then
            sleep 5
            continue
        fi

        "$DAEMON_BIN" --config "$CONFIG_FILE" --daemon >/dev/null 2>&1
        sleep 3
    done
else
    log_info "Error: $DAEMON_BIN not found or not executable."
fi
