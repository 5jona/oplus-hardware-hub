(() => {
  "use strict";

  window.OHH = window.OHH || {};

  function hasKsu() {
    return (
      typeof window.ksu !== "undefined" &&
      typeof window.ksu.exec === "function"
    );
  }

  let ksuCbCounter = 0;

  function ksuExec(cmd, options = {}) {
    return new Promise((resolve) => {
      if (hasKsu()) {
        const cbName = `__ohh_ksu_cb_${Date.now()}_${++ksuCbCounter}`;

        const timer = setTimeout(() => {
          if (window[cbName]) {
            delete window[cbName];
            resolve({ errno: -1, stdout: "", stderr: "Execution timeout" });
          }
        }, 8000);

        window[cbName] = (errno, stdout, stderr) => {
          clearTimeout(timer);
          delete window[cbName];
          resolve({
            errno: Number(errno) || 0,
            stdout: String(stdout != null ? stdout : ""),
            stderr: String(stderr != null ? stderr : ""),
          });
        };

        try {
          window.ksu.exec(cmd, JSON.stringify(options), cbName);
        } catch (err) {
          clearTimeout(timer);
          delete window[cbName];
          resolve({ errno: -2, stdout: "", stderr: String(err) });
        }
      } else {
        // Fallback simulation for browser development preview
        console.warn("[Bridge] KernelSU API not found, returning simulated telemetry");
        resolve({
          errno: 0,
          stdout: JSON.stringify({
            governor_enabled: true,
            state: "ACTIVE_GOVERNED",
            charging: true,
            battery_level: 68,
            battery_temp_c: 33.5,
            current_ma: -8150,
            battery_voltage_mv: 4210,
            usb_voltage_mv: 9200,
            power_watts: 74.9,
            cool_down_level: 0,
            max_allowed_temp: 42.0,
            recovery_temp: 38.0,
            screen_on_full_speed: true,
            device_max_power_w: 67,
            adapter_max_power_w: 100,
            consecutive_overrides: 142
          }),
          stderr: ""
        });
      }
    });
  }

  window.OHH.Bridge = {
    hasKsu,
    ksuExec,
  };
})();
