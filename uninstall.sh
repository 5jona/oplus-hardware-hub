#!/system/bin/sh
#
# Module cleanup script executed during uninstallation.
#

pkill -9 -x "ohh" 2>/dev/null || true
