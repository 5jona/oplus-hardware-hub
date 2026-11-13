#!/system/bin/sh
#
# Early boot initialization script executed by KernelSU during post-fs-data.
#

MODDIR="${0%/*}"

# Ensure default config file exists
if [ ! -f "$MODDIR/config.conf" ]; then
    cat <<'EOF' > "$MODDIR/config.conf"
governor_enabled=1
max_allowed_temp=42.0
recovery_temp=38.0
screen_on_full_speed=1
active_interval_ms=500
idle_interval_ms=3000
EOF
    chmod 0644 "$MODDIR/config.conf"
fi
