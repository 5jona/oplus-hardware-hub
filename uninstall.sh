#!/system/bin/sh
#
# Module cleanup script executed during uninstallation.
#

pkill -9 -x "charge_governor" 2>/dev/null || true
