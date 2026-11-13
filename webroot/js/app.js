(() => {
  "use strict";

  const { Bridge, Config } = window.OHH;

  let activeConfig = { ...Config.DEFAULT_CONFIG };
  let pollTimer = null;

  function $(id) {
    return document.getElementById(id);
  }

  function formatWatts(w) {
    if (w <= 0) return "0.0 W";
    return `${w.toFixed(1)} W`;
  }

  function formatCurrent(ma) {
    if (ma < 0) {
      return `+${Math.abs(ma)} mA`;
    } else if (ma > 0) {
      return `-${ma} mA`;
    }
    return "0 mA";
  }

  function formatVoltage(mv) {
    if (mv <= 0) return "0.00 V";
    return `${(mv / 1000).toFixed(2)} V`;
  }

  function updateTelemetryUI(status) {
    if (!status) return;

    // Hero metrics
    $("val-power").textContent = formatWatts(status.power_watts);
    $("val-level").textContent = `${status.battery_level}%`;
    $("val-temp").textContent = `${status.battery_temp_c.toFixed(1)} °C`;
    $("val-current").textContent = formatCurrent(status.current_ma);
    $("val-vbat").textContent = formatVoltage(status.battery_voltage_mv);
    $("val-vusb").textContent = formatVoltage(status.usb_voltage_mv);

    // Temperature badge color
    const tempBadge = $("badge-temp");
    if (tempBadge) {
      if (status.battery_temp_c >= status.max_allowed_temp) {
        tempBadge.className = "badge badge-danger";
        tempBadge.textContent = "Throttled";
      } else if (status.battery_temp_c >= status.recovery_temp) {
        tempBadge.className = "badge badge-warning";
        tempBadge.textContent = "Warm";
      } else {
        tempBadge.className = "badge badge-success";
        tempBadge.textContent = "Optimal";
      }
    }

    // State badge
    const stateBadge = $("badge-state");
    const stateDot = $("dot-state");
    if (stateBadge) {
      switch (status.state) {
        case "ACTIVE_GOVERNED":
          stateBadge.textContent = "Full Speed (0 Gear Locked)";
          stateBadge.className = "badge badge-success";
          if (stateDot) stateDot.className = "status-dot dot-active";
          break;
        case "SAFETY_THROTTLED":
          stateBadge.textContent = "Safety Tripwire (OEM Control)";
          stateBadge.className = "badge badge-danger";
          if (stateDot) stateDot.className = "status-dot dot-warning";
          break;
        case "DISCHARGING":
          stateBadge.textContent = "Discharging / Unplugged";
          stateBadge.className = "badge badge-secondary";
          if (stateDot) stateDot.className = "status-dot dot-idle";
          break;
        case "DISABLED":
          stateBadge.textContent = "Governor Disabled";
          stateBadge.className = "badge badge-secondary";
          if (stateDot) stateDot.className = "status-dot dot-idle";
          break;
        default:
          stateBadge.textContent = status.state;
          stateBadge.className = "badge badge-secondary";
          if (stateDot) stateDot.className = "status-dot dot-idle";
      }
    }

    $("val-cooldown").textContent = `Level ${status.cool_down_level}`;
    $("val-device-pwr").textContent = `${status.device_max_power_w} W`;
    $("val-adapter-pwr").textContent =
      status.adapter_max_power_w > 0 ? `${status.adapter_max_power_w} W` : "Negotiating";
    $("val-overrides").textContent = status.consecutive_overrides;
  }

  function bindInputEvents() {
    const swGovernor = $("sw-governor");
    const swScreenOn = $("sw-screenon");
    const sliderMaxTemp = $("slider-maxtemp");
    const sliderRecTemp = $("slider-rectemp");
    const valMaxTemp = $("label-maxtemp");
    const valRecTemp = $("label-rectemp");
    const btnSave = $("btn-save");

    if (sliderMaxTemp) {
      sliderMaxTemp.addEventListener("input", (e) => {
        const val = parseFloat(e.target.value);
        if (valMaxTemp) valMaxTemp.textContent = `${val.toFixed(1)} °C`;
        if (parseFloat(sliderRecTemp.value) > val - 1.0) {
          sliderRecTemp.value = (val - 1.5).toFixed(1);
          if (valRecTemp) valRecTemp.textContent = `${sliderRecTemp.value} °C`;
        }
      });
    }

    if (sliderRecTemp) {
      sliderRecTemp.addEventListener("input", (e) => {
        const val = parseFloat(e.target.value);
        if (valRecTemp) valRecTemp.textContent = `${val.toFixed(1)} °C`;
        if (val > parseFloat(sliderMaxTemp.value) - 1.0) {
          sliderMaxTemp.value = (val + 1.5).toFixed(1);
          if (valMaxTemp) valMaxTemp.textContent = `${sliderMaxTemp.value} °C`;
        }
      });
    }

    if (btnSave) {
      btnSave.addEventListener("click", async () => {
        btnSave.disabled = true;
        btnSave.textContent = "Saving...";

        const newCfg = {
          governor_enabled: swGovernor.checked,
          screen_on_full_speed: swScreenOn.checked,
          max_allowed_temp: parseFloat(sliderMaxTemp.value),
          recovery_temp: parseFloat(sliderRecTemp.value),
          active_interval_ms: parseInt($("sel-interval").value, 10),
          idle_interval_ms: 3000,
        };

        const res = await Config.saveConfig(newCfg);
        btnSave.disabled = false;
        if (res.errno === 0) {
          btnSave.textContent = "Applied Successfully";
          activeConfig = newCfg;
          setTimeout(() => {
            btnSave.textContent = "Save & Apply Settings";
          }, 2000);
        } else {
          btnSave.textContent = "Save Failed";
          alert(`Failed to save configuration: ${res.stderr || "Unknown error"}`);
          setTimeout(() => {
            btnSave.textContent = "Save & Apply Settings";
          }, 2000);
        }
      });
    }
  }

  function applyConfigToUI(cfg) {
    if ($("sw-governor")) $("sw-governor").checked = cfg.governor_enabled;
    if ($("sw-screenon")) $("sw-screenon").checked = cfg.screen_on_full_speed;
    if ($("slider-maxtemp")) $("slider-maxtemp").value = cfg.max_allowed_temp;
    if ($("label-maxtemp"))
      $("label-maxtemp").textContent = `${cfg.max_allowed_temp.toFixed(1)} °C`;
    if ($("slider-rectemp")) $("slider-rectemp").value = cfg.recovery_temp;
    if ($("label-rectemp"))
      $("label-rectemp").textContent = `${cfg.recovery_temp.toFixed(1)} °C`;
    if ($("sel-interval")) $("sel-interval").value = String(cfg.active_interval_ms);
  }

  async function startTelemetryLoop() {
    if (pollTimer) clearInterval(pollTimer);

    const poll = async () => {
      const status = await Config.queryStatus();
      updateTelemetryUI(status);
    };

    await poll();
    pollTimer = setInterval(poll, 1500);
  }

  document.addEventListener("DOMContentLoaded", async () => {
    bindInputEvents();
    activeConfig = await Config.loadConfig();
    applyConfigToUI(activeConfig);
    await startTelemetryLoop();
  });
})();
