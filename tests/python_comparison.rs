use rsthrottled::{calc_icc_max_msr, calc_time_window_vars};
use serde::Deserialize;

#[derive(Deserialize)]
struct IccMaxTest {
    plane: String,
    amp: f64,
    expected_hex: String,
}
#[derive(Deserialize)]
struct TimeTest {
    input_sec: f64,
    mock_time_unit: f64,
    expected_y: u64,
    expected_z: u64,
}
#[derive(Deserialize)]
struct TruthData {
    undervolt: Vec<serde_json::Value>, // already handled
    iccmax: Vec<IccMaxTest>,
    time_windows: Vec<TimeTest>,
}
const JSON_PATH: &str = "tests/fixtures/truth_data.json";

#[test]
fn test_iccmax() {
    let data = std::fs::read_to_string(JSON_PATH).unwrap();
    let truth: TruthData = serde_json::from_str(&data).unwrap();

    for t in truth.iccmax {
        let actual = calc_icc_max_msr(&t.plane, t.amp);
        let expected = u64::from_str_radix(t.expected_hex.trim_start_matches("0x"), 16).unwrap();
        assert_eq!(actual, expected, "IccMax fail: {}A on {}", t.amp, t.plane);
    }
}

#[test]
fn test_time_windows() {
    let data = std::fs::read_to_string(JSON_PATH).unwrap();
    let truth: TruthData = serde_json::from_str(&data).unwrap();

    for t in truth.time_windows {
        let (y, z) = calc_time_window_vars(t.input_sec, t.mock_time_unit);
        assert_eq!(
            (y, z),
            (t.expected_y, t.expected_z),
            "Time fail: {}s",
            t.input_sec
        );
    }
}
