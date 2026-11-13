(() => {
  "use strict";

  window.OHH = window.OHH || {};

  const MODULE_PATH = "/data/adb/modules/oplus_hardware_hub";
  const CONFIG_PATH = `${MODULE_PATH}/config.conf`;
  const DAEMON_BIN = `${MODULE_PATH}/bin/ohh`;

  const DEFAULT_CONFIG = {
    governor_enabled: true,
    max_allowed_temp: 42.0,
    recovery_temp: 38.0,
    screen_on_full_speed: true,
    charge_limit_enabled: true,
    charge_limit_level: 80,
    active_interval_ms: 500,
    idle_interval_ms: 3000,
  };

  async function fetchSystemChargeLimit() {
    const swRes = await window.OHH.Bridge.ksuExec(
      "cmd settings get system regular_charge_protection_switch_state 2>/dev/null || settings get system regular_charge_protection_switch_state 2>/dev/null"
    );
    const lvlRes = await window.OHH.Bridge.ksuExec(
      "cmd settings get system regular_charge_protection_open_power_level 2>/dev/null || settings get system regular_charge_protection_open_power_level 2>/dev/null"
    );

    const result = {};
    if (swRes && swRes.errno === 0 && swRes.stdout.trim() && swRes.stdout.trim() !== "null") {
      result.charge_limit_enabled = swRes.stdout.trim() === "1";
    }
    if (lvlRes && lvlRes.errno === 0 && lvlRes.stdout.trim() && lvlRes.stdout.trim() !== "null") {
      const lvl = parseInt(lvlRes.stdout.trim(), 10);
      if (!isNaN(lvl) && lvl >= 80 && lvl <= 100) {
        result.charge_limit_level = lvl;
      }
    }
    return result;
  }

  async function loadConfig() {
    const res = await window.OHH.Bridge.ksuExec(`cat "${CONFIG_PATH}" 2>/dev/null`);
    const cfg = { ...DEFAULT_CONFIG };
    if (res.errno === 0 && res.stdout.trim()) {
      for (const line of res.stdout.split("\n")) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith("#")) continue;

        const idx = trimmed.indexOf("=");
        if (idx === -1) continue;

        const key = trimmed.slice(0, idx).trim();
        const val = trimmed.slice(idx + 1).trim();

        if (key === "governor_enabled") {
          cfg.governor_enabled = val === "1" || val.toLowerCase() === "true";
        } else if (key === "max_allowed_temp") {
          const num = parseFloat(val);
          if (!isNaN(num)) cfg.max_allowed_temp = num;
        } else if (key === "recovery_temp") {
          const num = parseFloat(val);
          if (!isNaN(num)) cfg.recovery_temp = num;
        } else if (key === "screen_on_full_speed") {
          cfg.screen_on_full_speed = val === "1" || val.toLowerCase() === "true";
        } else if (key === "charge_limit_enabled") {
          cfg.charge_limit_enabled = val === "1" || val.toLowerCase() === "true";
        } else if (key === "charge_limit_level") {
          const num = parseInt(val, 10);
          if (!isNaN(num) && num >= 80 && num <= 100) cfg.charge_limit_level = num;
        } else if (key === "active_interval_ms") {
          const num = parseInt(val, 10);
          if (!isNaN(num)) cfg.active_interval_ms = num;
        } else if (key === "idle_interval_ms") {
          const num = parseInt(val, 10);
          if (!isNaN(num)) cfg.idle_interval_ms = num;
        }
      }
    }

    // Prefer live system setting if available
    try {
      const sysLimit = await fetchSystemChargeLimit();
      Object.assign(cfg, sysLimit);
    } catch (e) {
      console.warn("[Config] Could not fetch system charge limit:", e);
    }

    return cfg;
  }

  async function saveConfig(cfg) {
    const content = [
      "# Configuration for OPlus Hardware Hub",
      `governor_enabled=${cfg.governor_enabled ? 1 : 0}`,
      `max_allowed_temp=${cfg.max_allowed_temp.toFixed(1)}`,
      `recovery_temp=${cfg.recovery_temp.toFixed(1)}`,
      `screen_on_full_speed=${cfg.screen_on_full_speed ? 1 : 0}`,
      `charge_limit_enabled=${cfg.charge_limit_enabled ? 1 : 0}`,
      `charge_limit_level=${cfg.charge_limit_level}`,
      `active_interval_ms=${cfg.active_interval_ms}`,
      `idle_interval_ms=${cfg.idle_interval_ms}`,
    ].join("\n");

    const cmdFile = `cat <<'EOF' > "${CONFIG_PATH}"\n${content}\nEOF\nchmod 0644 "${CONFIG_PATH}"`;
    const resFile = await window.OHH.Bridge.ksuExec(cmdFile);

    // Apply directly to system settings
    const swVal = cfg.charge_limit_enabled ? 1 : 0;
    const lvlVal = cfg.charge_limit_level;
    const cmdSettings = `cmd settings put system regular_charge_protection_switch_state ${swVal} 2>/dev/null || settings put system regular_charge_protection_switch_state ${swVal} 2>/dev/null; cmd settings put system regular_charge_protection_open_power_level ${lvlVal} 2>/dev/null || settings put system regular_charge_protection_open_power_level ${lvlVal} 2>/dev/null`;
    await window.OHH.Bridge.ksuExec(cmdSettings);

    return resFile;
  }

  async function queryStatus() {
    const cmd = `"${DAEMON_BIN}" --config "${CONFIG_PATH}" --status 2>/dev/null`;
    const res = await window.OHH.Bridge.ksuExec(cmd);
    if (res.errno === 0 && res.stdout) {
      try {
        return JSON.parse(res.stdout);
      } catch (err) {
        console.error("[Config] JSON parse error:", err, res.stdout);
      }
    }
    return null;
  }

  window.OHH.Config = {
    MODULE_PATH,
    CONFIG_PATH,
    DAEMON_BIN,
    DEFAULT_CONFIG,
    loadConfig,
    saveConfig,
    queryStatus,
  };
})();
