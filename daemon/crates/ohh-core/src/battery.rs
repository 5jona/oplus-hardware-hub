use crate::error::GovernorError;
use std::fs;
use std::path::Path;

/// Telemetry metrics for power supply and battery hardware subsystem.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BatteryTelemetry {
    /// Whether the device is currently connected to a power supply.
    pub charging: bool,
    /// Battery State of Charge in percentage.
    pub battery_level: i32,
    /// Battery temperature in degrees Celsius.
    pub battery_temp_c: f32,
    /// Battery current in mA (negative when charging).
    pub current_ma: i32,
    /// Battery cell/pack voltage in mV.
    pub battery_voltage_mv: i32,
    /// Input bus voltage in mV.
    pub usb_voltage_mv: i32,
    /// Estimated charging power delivered in Watts.
    pub power_watts: f32,
    /// Current MMS thermal cool_down throttling level (0 = unconstrained).
    pub cool_down_level: i32,
    /// Maximum rated power capability of the device in Watts.
    pub device_max_power_w: i32,
    /// Maximum negotiated adapter power in Watts.
    pub adapter_max_power_w: i32,
    /// Fast charging protocol identifier.
    pub protocol_type: i32,
    /// Fast charging physical mode identifier.
    pub fast_chg_type: i32,
}

/// Interface to power supply and OPLUS charging sysfs nodes.
#[derive(Debug, Clone)]
pub struct BatterySubsystem {
    path_batt_temp: &'static str,
    path_batt_current: &'static str,
    path_batt_voltage: &'static str,
    path_batt_capacity: &'static str,
    path_batt_status: &'static str,
    path_usb_online: &'static str,
    path_usb_voltage: &'static str,
    path_cool_down: &'static str,
    path_device_power: &'static str,
    path_adapter_power: &'static str,
    path_protocol_type: &'static str,
    path_fast_chg_type: &'static str,
}

impl Default for BatterySubsystem {
    fn default() -> Self {
        Self::new()
    }
}

impl BatterySubsystem {
    /// Constructs a new battery subsystem interface pointing to standard OPLUS sysfs paths.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            path_batt_temp: "/sys/class/power_supply/battery/temp",
            path_batt_current: "/sys/class/power_supply/battery/current_now",
            path_batt_voltage: "/sys/class/power_supply/battery/voltage_now",
            path_batt_capacity: "/sys/class/power_supply/battery/capacity",
            path_batt_status: "/sys/class/power_supply/battery/status",
            path_usb_online: "/sys/class/power_supply/usb/online",
            path_usb_voltage: "/sys/class/power_supply/usb/voltage_now",
            path_cool_down: "/sys/class/oplus_chg/battery/cool_down",
            path_device_power: "/sys/class/oplus_chg/common/device_power",
            path_adapter_power: "/sys/class/oplus_chg/common/adapter_power",
            path_protocol_type: "/sys/class/oplus_chg/common/protocol_type",
            path_fast_chg_type: "/sys/class/oplus_chg/usb/fast_chg_type",
        }
    }

    fn read_trimmed_string(&self, path: &str) -> Option<String> {
        fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    }

    fn read_i32(&self, path: &str) -> Option<i32> {
        self.read_trimmed_string(path)
            .and_then(|s| s.parse::<i32>().ok())
    }

    /// Reads and normalizes all available hardware telemetry.
    #[must_use]
    pub fn read_telemetry(&self) -> BatteryTelemetry {
        let mut telemetry = BatteryTelemetry::default();

        let usb_online = self.read_i32(self.path_usb_online).unwrap_or(0) == 1;
        let batt_status = self
            .read_trimmed_string(self.path_batt_status)
            .unwrap_or_default();
        let status_charging = batt_status.eq_ignore_ascii_case("Charging")
            || batt_status.eq_ignore_ascii_case("Full");
        telemetry.charging = usb_online || status_charging;

        telemetry.battery_level = self.read_i32(self.path_batt_capacity).unwrap_or(0);

        let raw_temp = self.read_i32(self.path_batt_temp).unwrap_or(250);
        telemetry.battery_temp_c = (raw_temp as f32) / 10.0;

        let raw_current = self.read_i32(self.path_batt_current).unwrap_or(0);
        telemetry.current_ma = if raw_current.abs() > 100_000 {
            raw_current / 1000
        } else {
            raw_current
        };

        let raw_batt_voltage = self.read_i32(self.path_batt_voltage).unwrap_or(0);
        telemetry.battery_voltage_mv = if raw_batt_voltage > 100_000 {
            raw_batt_voltage / 1000
        } else {
            raw_batt_voltage
        };

        let raw_usb_voltage = self.read_i32(self.path_usb_voltage).unwrap_or(0);
        telemetry.usb_voltage_mv = if raw_usb_voltage > 100_000 {
            raw_usb_voltage / 1000
        } else {
            raw_usb_voltage
        };

        telemetry.cool_down_level = self.read_i32(self.path_cool_down).unwrap_or(0);

        let dev_pwr_mw = self.read_i32(self.path_device_power).unwrap_or(67000);
        telemetry.device_max_power_w = dev_pwr_mw / 1000;

        let adapter_pwr_mw = self.read_i32(self.path_adapter_power).unwrap_or(0);
        telemetry.adapter_max_power_w = adapter_pwr_mw / 1000;

        telemetry.protocol_type = self.read_i32(self.path_protocol_type).unwrap_or(0);
        telemetry.fast_chg_type = self.read_i32(self.path_fast_chg_type).unwrap_or(0);

        if telemetry.charging && telemetry.usb_voltage_mv > 4000 && telemetry.current_ma < 0 {
            let abs_current_a = (telemetry.current_ma.abs() as f32) / 1000.0;
            let cell_voltage_v = (telemetry.battery_voltage_mv as f32) / 1000.0;

            let estimated_pwr = if telemetry.usb_voltage_mv >= 8000 {
                let vbus_v = (telemetry.usb_voltage_mv as f32) / 1000.0;
                vbus_v * (abs_current_a / 2.0)
            } else {
                cell_voltage_v * abs_current_a
            };
            telemetry.power_watts = (estimated_pwr * 10.0).round() / 10.0;
        } else {
            telemetry.power_watts = 0.0;
        }

        telemetry
    }

    /// Sets the MMS cool_down level.
    pub fn write_cool_down(&self, level: i32) -> Result<(), GovernorError> {
        let content = format!("{level}\n");
        fs::write(self.path_cool_down, content).map_err(GovernorError::Io)
    }

    /// Checks if the cool_down sysfs control interface exists.
    #[must_use]
    pub fn is_supported(&self) -> bool {
        Path::new(self.path_cool_down).exists()
    }

    fn run_settings_cmd(args: &[&str]) -> Option<String> {
        let bin = if Path::new("/system/bin/cmd").exists() {
            "/system/bin/cmd"
        } else {
            "cmd"
        };
        let mut full_args = vec!["settings"];
        full_args.extend_from_slice(args);
        let out = std::process::Command::new(bin)
            .args(full_args)
            .output()
            .ok()?;
        if out.status.success() {
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Queries the current system charge limit settings from ColorOS SettingsProvider.
    #[must_use]
    pub fn query_system_charge_limit(&self) -> Option<(bool, u32)> {
        let sw_str =
            Self::run_settings_cmd(&["get", "system", "regular_charge_protection_switch_state"])?;
        let lvl_str = Self::run_settings_cmd(&[
            "get",
            "system",
            "regular_charge_protection_open_power_level",
        ])?;

        if sw_str == "null" || lvl_str == "null" {
            return None;
        }

        let enabled = sw_str == "1";
        let level = lvl_str.parse::<u32>().ok()?;
        Some((enabled, level))
    }

    /// Enforces the system charge limit settings in ColorOS SettingsProvider.
    pub fn apply_system_charge_limit(&self, enabled: bool, level: u32) {
        let sw_val = if enabled { "1" } else { "0" };
        let lvl_val = level.to_string();

        let _ = Self::run_settings_cmd(&[
            "put",
            "system",
            "regular_charge_protection_switch_state",
            sw_val,
        ]);
        let _ = Self::run_settings_cmd(&[
            "put",
            "system",
            "regular_charge_protection_open_power_level",
            &lvl_val,
        ]);
    }
}
