use governor_core::{ChargeGovernor, GovernorConfig, GovernorState};
use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

fn print_usage() {
    println!("OnePlus Smart FastCharge & Thermal Governor");
    println!("Usage: charge_governor [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --config <path>   Specify configuration file path");
    println!("  --status          Print current hardware and governor telemetry in JSON and exit");
    println!("  --apply-once      Evaluate telemetry, apply overrides once, and exit");
    println!("  --daemon          Run background continuous governor loop (default)");
    println!("  --help, -h        Print this help message");
    println!("  --version, -v     Print version information");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config_path = PathBuf::from("/data/adb/modules/oplus_smart_charge/config.conf");
    let mut mode_status = false;
    let mut mode_apply_once = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                if i + 1 < args.len() {
                    config_path = PathBuf::from(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("Error: --config requires a file path argument");
                    std::process::exit(1);
                }
            }
            "--status" => {
                mode_status = true;
            }
            "--apply-once" => {
                mode_apply_once = true;
            }
            "--daemon" => {
                // Default operational mode
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--version" | "-v" => {
                println!("charge_governor v{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            unknown => {
                eprintln!("Error: Unknown argument '{unknown}'");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let mut governor = ChargeGovernor::new();
    let config = GovernorConfig::load_from_file(&config_path);

    if mode_status {
        println!("{}", governor.status_json(&config));
        return;
    }

    if mode_apply_once {
        let (state, telemetry) = governor.evaluate_and_step(&config);
        let state_str = state.as_str();
        let temp = telemetry.battery_temp_c;
        let curr = telemetry.current_ma;
        let pwr = telemetry.power_watts;
        let cd = telemetry.cool_down_level;

        println!(
            "State: {state_str}, Temp: {temp:.1}C, Current: {curr}mA, Power: {pwr:.1}W, CoolDown: {cd}"
        );
        return;
    }

    let mut active_config = config;
    let mut last_config_check = Instant::now();

    loop {
        if last_config_check.elapsed() >= Duration::from_secs(3) {
            active_config = GovernorConfig::load_from_file(&config_path);
            last_config_check = Instant::now();
        }

        let (state, _telemetry) = governor.evaluate_and_step(&active_config);

        let sleep_duration = match state {
            GovernorState::ActiveGoverned => {
                Duration::from_millis(active_config.active_interval_ms)
            }
            GovernorState::SafetyThrottled => Duration::from_millis(1000),
            GovernorState::Discharging | GovernorState::Disabled => {
                Duration::from_millis(active_config.idle_interval_ms)
            }
        };

        thread::sleep(sleep_duration);
    }
}
