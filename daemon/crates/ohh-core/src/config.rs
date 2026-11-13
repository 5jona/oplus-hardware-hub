use std::fs;
use std::path::{Path, PathBuf};

/// Runtime configuration parameters for the thermal governor.
#[derive(Debug, Clone, PartialEq)]
pub struct GovernorConfig {
    /// Global master enable switch for the governor loop.
    pub governor_enabled: bool,
    /// Maximum battery temperature in Celsius allowed before thermal tripwire disengages.
    pub max_allowed_temp: f32,
    /// Recovery temperature threshold in Celsius for re-engaging unconstrained fast charging.
    pub recovery_temp: f32,
    /// Whether to maintain unconstrained charging while the display is active.
    pub screen_on_full_speed: bool,
    /// Polling and enforcement period in milliseconds when charging is detected.
    pub active_interval_ms: u64,
    /// Polling period in milliseconds when the device is discharging or idle.
    pub idle_interval_ms: u64,
    /// Underlying file path from which the configuration was loaded.
    pub config_path: PathBuf,
}

impl Default for GovernorConfig {
    fn default() -> Self {
        Self {
            governor_enabled: true,
            max_allowed_temp: 42.0,
            recovery_temp: 38.0,
            screen_on_full_speed: true,
            active_interval_ms: 500,
            idle_interval_ms: 3000,
            config_path: PathBuf::from("/data/adb/modules/oplus_hardware_hub/config.conf"),
        }
    }
}

impl GovernorConfig {
    /// Loads configuration parameters from disk, substituting missing or invalid keys with defaults.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let Ok(content) = fs::read_to_string(&path_buf) else {
            return Self {
                config_path: path_buf,
                ..Default::default()
            };
        };

        let mut config = Self::parse_str(&content);
        config.config_path = path_buf;
        config
    }

    /// Parses configuration parameters from an in-memory string.
    #[must_use]
    pub fn parse_str(content: &str) -> Self {
        let mut config = Self::default();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let Some((key, val)) = trimmed.split_once('=') else {
                continue;
            };

            let k = key.trim();
            let v = val.trim();

            match k {
                "governor_enabled" => {
                    config.governor_enabled = Self::parse_bool(v, config.governor_enabled);
                }
                "max_allowed_temp" => {
                    if let Ok(temp) = v.parse::<f32>()
                        && (30.0..=55.0).contains(&temp)
                    {
                        config.max_allowed_temp = temp;
                    }
                }
                "recovery_temp" => {
                    if let Ok(temp) = v.parse::<f32>()
                        && (25.0..=50.0).contains(&temp)
                    {
                        config.recovery_temp = temp;
                    }
                }
                "screen_on_full_speed" => {
                    config.screen_on_full_speed = Self::parse_bool(v, config.screen_on_full_speed);
                }
                "active_interval_ms" => {
                    if let Ok(ms) = v.parse::<u64>()
                        && (100..=10000).contains(&ms)
                    {
                        config.active_interval_ms = ms;
                    }
                }
                "idle_interval_ms" => {
                    if let Ok(ms) = v.parse::<u64>()
                        && (500..=60000).contains(&ms)
                    {
                        config.idle_interval_ms = ms;
                    }
                }
                _ => {}
            }
        }

        if config.recovery_temp > config.max_allowed_temp {
            config.recovery_temp = config.max_allowed_temp - 2.0;
        }

        config
    }

    fn parse_bool(val: &str, default: bool) -> bool {
        match val.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let text = r#"
            # Governor configuration
            governor_enabled=1
            max_allowed_temp=42.5
            recovery_temp=39.0
            screen_on_full_speed=0
            active_interval_ms=600
            idle_interval_ms=5000
        "#;
        let config = GovernorConfig::parse_str(text);
        assert!(config.governor_enabled);
        assert!((config.max_allowed_temp - 42.5).abs() < f32::EPSILON);
        assert!((config.recovery_temp - 39.0).abs() < f32::EPSILON);
        assert!(!config.screen_on_full_speed);
        assert_eq!(config.active_interval_ms, 600);
        assert_eq!(config.idle_interval_ms, 5000);
    }

    #[test]
    fn test_recovery_temp_clamping() {
        let text = r#"
            max_allowed_temp=35.0
            recovery_temp=40.0
        "#;
        let config = GovernorConfig::parse_str(text);
        assert!((config.max_allowed_temp - 35.0).abs() < f32::EPSILON);
        assert!((config.recovery_temp - 33.0).abs() < f32::EPSILON);
    }
}
