#!/system/bin/sh
#
# Module installation and permission setup script for KernelSU / Magisk.
#

SKIPUNZIP=0

ui_print "- Installing OPlus Hardware Hub..."

# Verify architecture
if [ "$ARCH" != "arm64" ]; then
    abort "! Unsupported architecture: $ARCH. This module only supports arm64 devices."
fi

# Verify OPLUS charging subsystem presence
if [ ! -d "/sys/class/oplus_chg/battery" ]; then
    ui_print "! Warning: /sys/class/oplus_chg/battery not detected."
    ui_print "! Ensure your device is a OnePlus device running ColorOS/OxygenOS or compatible kernel."
fi

ui_print "- Setting executable permissions..."
set_perm "$MODPATH/bin/ohh" 0 0 0755
set_perm "$MODPATH/service.sh" 0 0 0755
set_perm "$MODPATH/post-fs-data.sh" 0 0 0755
set_perm "$MODPATH/uninstall.sh" 0 0 0755

if [ -f "$MODPATH/config.conf" ]; then
    set_perm "$MODPATH/config.conf" 0 0 0644
fi

ui_print "- Installation completed successfully."
