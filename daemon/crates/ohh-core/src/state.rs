use crate::battery::{BatterySubsystem, BatteryTelemetry};
use crate::config::GovernorConfig;

/// Operating states of the thermal governor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GovernorState {
    /// Device is discharging or not connected to a validated charger.
    #[default]
    Discharging,
    /// Fast charging active under acceptable thermal threshold; 0 gear enforced.
    ActiveGoverned,
    /// Safety thermal threshold reached; governor released control to OEM thermal daemon.
    SafetyThrottled,
    /// Governor disabled by user configuration.
    Disabled,
}

impl GovernorState {
    /// Returns the canonical uppercase string representation of the state.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Discharging => "DISCHARGING",
            Self::ActiveGoverned => "ACTIVE_GOVERNED",
            Self::SafetyThrottled => "SAFETY_THROTTLED",
            Self::Disabled => "DISABLED",
        }
    }
}

/// Core charge governor controller.
#[derive(Debug)]
pub struct ChargeGovernor {
    battery: BatterySubsystem,
    current_state: GovernorState,
    consecutive_overrides: u64,
}

impl Default for ChargeGovernor {
    fn default() -> Self {
        Self::new()
    }
}

impl ChargeGovernor {
    /// Constructs a new governor instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            battery: BatterySubsystem::new(),
            current_state: GovernorState::Discharging,
            consecutive_overrides: 0,
        }
    }

    /// Evaluates hardware telemetry and applies governance rules.
    pub fn evaluate_and_step(
        &mut self,
        config: &GovernorConfig,
    ) -> (GovernorState, BatteryTelemetry) {
        let telemetry = self.battery.read_telemetry();
        let state = self.step_with_telemetry(config, &telemetry);
        (state, telemetry)
    }

    /// Evaluates a telemetry snapshot and executes state transitions.
    pub fn step_with_telemetry(
        &mut self,
        config: &GovernorConfig,
        telemetry: &BatteryTelemetry,
    ) -> GovernorState {
        if !config.governor_enabled {
            self.current_state = GovernorState::Disabled;
            return self.current_state;
        }

        if !telemetry.charging {
            self.current_state = GovernorState::Discharging;
            self.consecutive_overrides = 0;
            return self.current_state;
        }

        match self.current_state {
            GovernorState::SafetyThrottled => {
                if telemetry.battery_temp_c <= config.recovery_temp {
                    self.current_state = GovernorState::ActiveGoverned;
                    let _ = self.battery.write_cool_down(0);
                    self.consecutive_overrides += 1;
                }
            }
            _ => {
                if telemetry.battery_temp_c >= config.max_allowed_temp {
                    self.current_state = GovernorState::SafetyThrottled;
                } else {
                    self.current_state = GovernorState::ActiveGoverned;
                    if telemetry.cool_down_level != 0 {
                        let _ = self.battery.write_cool_down(0);
                        self.consecutive_overrides += 1;
                    }
                }
            }
        }

        self.current_state
    }

    /// Synchronizes the charge limit settings with ColorOS system settings.
    pub fn sync_charge_limit(&self, config: &GovernorConfig) {
        self.battery
            .apply_system_charge_limit(config.charge_limit_enabled, config.charge_limit_level);
    }

    /// Generates structured status JSON string for external status queries.
    pub fn status_json(&mut self, config: &GovernorConfig) -> String {
        let (state, telemetry) = self.evaluate_and_step(config);
        let gov_enabled = config.governor_enabled;
        let state_str = state.as_str();
        let charging = telemetry.charging;
        let batt_level = telemetry.battery_level;
        let batt_temp = telemetry.battery_temp_c;
        let curr_ma = telemetry.current_ma;
        let vbat_mv = telemetry.battery_voltage_mv;
        let vusb_mv = telemetry.usb_voltage_mv;
        let power_w = telemetry.power_watts;
        let cool_down = telemetry.cool_down_level;
        let max_temp = config.max_allowed_temp;
        let rec_temp = config.recovery_temp;
        let screen_on = config.screen_on_full_speed;
        let chg_lim_enabled = config.charge_limit_enabled;
        let chg_lim_level = config.charge_limit_level;
        let dev_pwr = telemetry.device_max_power_w;
        let adp_pwr = telemetry.adapter_max_power_w;
        let overrides = self.consecutive_overrides;

        format!(
            "{{\n  \
              \"governor_enabled\": {gov_enabled},\n  \
              \"state\": \"{state_str}\",\n  \
              \"charging\": {charging},\n  \
              \"battery_level\": {batt_level},\n  \
              \"battery_temp_c\": {batt_temp:.1},\n  \
              \"current_ma\": {curr_ma},\n  \
              \"battery_voltage_mv\": {vbat_mv},\n  \
              \"usb_voltage_mv\": {vusb_mv},\n  \
              \"power_watts\": {power_w:.1},\n  \
              \"cool_down_level\": {cool_down},\n  \
              \"max_allowed_temp\": {max_temp:.1},\n  \
              \"recovery_temp\": {rec_temp:.1},\n  \
              \"screen_on_full_speed\": {screen_on},\n  \
              \"charge_limit_enabled\": {chg_lim_enabled},\n  \
              \"charge_limit_level\": {chg_lim_level},\n  \
              \"device_max_power_w\": {dev_pwr},\n  \
              \"adapter_max_power_w\": {adp_pwr},\n  \
              \"consecutive_overrides\": {overrides}\n\
            }}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governor_state_transitions() {
        let mut governor = ChargeGovernor::new();
        let config = GovernorConfig {
            governor_enabled: true,
            max_allowed_temp: 42.0,
            recovery_temp: 38.0,
            ..Default::default()
        };

        // 1. Initial discharging state
        let mut telemetry = BatteryTelemetry {
            charging: false,
            battery_temp_c: 32.0,
            cool_down_level: 0,
            ..Default::default()
        };
        assert_eq!(
            governor.step_with_telemetry(&config, &telemetry),
            GovernorState::Discharging
        );

        // 2. Charger attached, acceptable temperature
        telemetry.charging = true;
        telemetry.cool_down_level = 5;
        assert_eq!(
            governor.step_with_telemetry(&config, &telemetry),
            GovernorState::ActiveGoverned
        );

        // 3. Temperature exceeds max limit -> SafetyThrottled
        telemetry.battery_temp_c = 42.5;
        assert_eq!(
            governor.step_with_telemetry(&config, &telemetry),
            GovernorState::SafetyThrottled
        );

        // 4. In between max and recovery -> still SafetyThrottled (hysteresis)
        telemetry.battery_temp_c = 39.5;
        assert_eq!(
            governor.step_with_telemetry(&config, &telemetry),
            GovernorState::SafetyThrottled
        );

        // 5. Temperature drops below recovery threshold -> ActiveGoverned
        telemetry.battery_temp_c = 37.5;
        assert_eq!(
            governor.step_with_telemetry(&config, &telemetry),
            GovernorState::ActiveGoverned
        );

        // 6. Charger disconnected
        telemetry.charging = false;
        assert_eq!(
            governor.step_with_telemetry(&config, &telemetry),
            GovernorState::Discharging
        );
    }

    #[test]
    fn test_governor_disabled() {
        let mut governor = ChargeGovernor::new();
        let config = GovernorConfig {
            governor_enabled: false,
            ..Default::default()
        };
        let telemetry = BatteryTelemetry {
            charging: true,
            battery_temp_c: 30.0,
            ..Default::default()
        };
        assert_eq!(
            governor.step_with_telemetry(&config, &telemetry),
            GovernorState::Disabled
        );
    }

    #[test]
    fn test_status_json_charge_limit() {
        let mut governor = ChargeGovernor::new();
        let config = GovernorConfig {
            charge_limit_enabled: true,
            charge_limit_level: 85,
            ..Default::default()
        };
        let json = governor.status_json(&config);
        assert!(json.contains("\"charge_limit_enabled\": true"));
        assert!(json.contains("\"charge_limit_level\": 85"));
    }
}
