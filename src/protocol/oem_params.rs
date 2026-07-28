// ponytail: OEM parameter catalog for Vehicle, Motor, and Control tabs with safety bounds and serde support.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OemCategory {
    Vehicle,
    Motor,
    Control,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    U8,
    U16,
    Bool,
    ReadOnlyText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub addr: u8,
    pub key: &'static str,
    pub label: &'static str,
    pub category: OemCategory,
    pub val_type: ValueType,
    pub is_read_only: bool,
    pub is_critical: bool,
    pub min_val: u16,
    pub max_val: u16,
    pub default_val: u16,
    pub unit: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParamProfile {
    pub vehicle_name: String,
    pub values: std::collections::BTreeMap<u8, u16>,
    pub text_values: std::collections::BTreeMap<u8, String>,
}

pub fn get_all_param_defs() -> &'static [ParamDef] {
    &PARAM_DEFS
}

pub static PARAM_DEFS: [ParamDef; 61] = [

    // --- VEHICLE TAB: Read-Only Fields ---
    ParamDef { addr: 0x00, key: "module_name", label: "Module Name", category: OemCategory::Vehicle, val_type: ValueType::ReadOnlyText, is_read_only: true, is_critical: false, min_val: 0, max_val: 0, default_val: 0, unit: "" },
    ParamDef { addr: 0x08, key: "user_name", label: "User Name", category: OemCategory::Vehicle, val_type: ValueType::ReadOnlyText, is_read_only: true, is_critical: false, min_val: 0, max_val: 0, default_val: 0, unit: "" },
    ParamDef { addr: 0x0C, key: "serial_number", label: "Serial Number", category: OemCategory::Vehicle, val_type: ValueType::ReadOnlyText, is_read_only: true, is_critical: false, min_val: 0, max_val: 0, default_val: 0, unit: "" },
    ParamDef { addr: 0x10, key: "sw_version", label: "Software Version", category: OemCategory::Vehicle, val_type: ValueType::ReadOnlyText, is_read_only: true, is_critical: false, min_val: 0, max_val: 0, default_val: 0, unit: "" },
    ParamDef { addr: 0x17, key: "ctrl_volt", label: "Controller Volt", category: OemCategory::Vehicle, val_type: ValueType::U16, is_read_only: true, is_critical: false, min_val: 0, max_val: 612, default_val: 72, unit: "V" },
    ParamDef { addr: 0x1D, key: "hall_galv", label: "Hall Galvanometer", category: OemCategory::Vehicle, val_type: ValueType::U16, is_read_only: true, is_critical: false, min_val: 0, max_val: 1000, default_val: 525, unit: "A" },
    ParamDef { addr: 0x1F, key: "phase_curr_max_ad", label: "PhaseCurr Max AD", category: OemCategory::Vehicle, val_type: ValueType::U16, is_read_only: true, is_critical: false, min_val: 409, max_val: 2048, default_val: 380, unit: "AD" },

    // --- VEHICLE TAB: Editable Numeric Fields ---
    ParamDef { addr: 0x19, key: "low_volt", label: "Low Volt", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: true, min_val: 0, max_val: 200, default_val: 52, unit: "V" },
    ParamDef { addr: 0x1B, key: "over_volt", label: "Over Volt", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: true, min_val: 0, max_val: 200, default_val: 68, unit: "V" },
    ParamDef { addr: 0x25, key: "curr_percent", label: "Current Percent", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: true, min_val: 20, max_val: 100, default_val: 100, unit: "%" },
    ParamDef { addr: 0x26, key: "battry_limit", label: "Battry Limit", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: true, min_val: 20, max_val: 100, default_val: 100, unit: "%" },
    ParamDef { addr: 0x38, key: "id_angle", label: "Identification Angle", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: true, min_val: 0, max_val: 255, default_val: 85, unit: "deg" },
    ParamDef { addr: 0x5C, key: "tps_low_err", label: "TPS Low Err", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 20, default_val: 0, unit: "%" },
    ParamDef { addr: 0x5D, key: "tps_high_err", label: "TPS High Err", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 80, max_val: 100, default_val: 95, unit: "%" },
    ParamDef { addr: 0x5F, key: "tps_type", label: "TPS Type", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 3, default_val: 1, unit: "" },
    ParamDef { addr: 0x60, key: "tps_dead_low", label: "TPS Dead Low", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 80, default_val: 25, unit: "%" },
    ParamDef { addr: 0x61, key: "tps_dead_high", label: "TPS Dead High", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 120, max_val: 200, default_val: 100, unit: "%" },
    ParamDef { addr: 0x62, key: "tps_fwd_map", label: "TPS Fwd MAP", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 100, default_val: 60, unit: "%" },
    ParamDef { addr: 0x63, key: "tps_rev_map", label: "TPS Rev MAP", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 100, default_val: 20, unit: "%" },
    ParamDef { addr: 0x64, key: "brake_type", label: "Brake Type", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 3, default_val: 1, unit: "" },
    ParamDef { addr: 0x65, key: "brake_dead_low", label: "Brake Dead Low", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 80, default_val: 10, unit: "%" },
    ParamDef { addr: 0x66, key: "brake_dead_high", label: "Brake Dead High", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 120, max_val: 200, default_val: 95, unit: "%" },
    ParamDef { addr: 0x69, key: "max_output_fre", label: "Max Output Fre", category: OemCategory::Vehicle, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 1000, default_val: 1000, unit: "Hz" },
    ParamDef { addr: 0x6B, key: "max_speed", label: "Max Speed", category: OemCategory::Vehicle, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 60000, default_val: 6000, unit: "RPM" },
    ParamDef { addr: 0x6D, key: "max_fwd_speed_pct", label: "Max Fwd Speed %", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 30, max_val: 100, default_val: 100, unit: "%" },
    ParamDef { addr: 0x6E, key: "max_rev_speed_pct", label: "Max Rev Speed %", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 20, max_val: 100, default_val: 80, unit: "%" },
    ParamDef { addr: 0x70, key: "midspeed_forw", label: "MidSpeed Forw", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 100, default_val: 100, unit: "%" },
    ParamDef { addr: 0x71, key: "midspeed_rev", label: "MidSpeed Rev Speed", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 100, default_val: 30, unit: "%" },
    ParamDef { addr: 0x72, key: "lowspeed_forw", label: "LowSpeed Forw", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 100, default_val: 15, unit: "%" },
    ParamDef { addr: 0x73, key: "lowspeed_rev", label: "LowSpeed Rev Speed", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 100, default_val: 15, unit: "%" },
    ParamDef { addr: 0x74, key: "three_speed", label: "Three Speed", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 3, default_val: 1, unit: "" },
    ParamDef { addr: 0x75, key: "pwm_frequency", label: "PWM frequency", category: OemCategory::Vehicle, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 10, max_val: 20, default_val: 20, unit: "kHz" },

    // --- MOTOR TAB: Editable Fields ---
    ParamDef { addr: 0x20, key: "motor_nom_curr", label: "Motor Nominal Curr", category: OemCategory::Motor, val_type: ValueType::U16, is_read_only: false, is_critical: true, min_val: 0, max_val: 1000, default_val: 90, unit: "A" },
    ParamDef { addr: 0x40, key: "motor_poles", label: "Motor Poles", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: true, min_val: 2, max_val: 128, default_val: 8, unit: "" },
    ParamDef { addr: 0x41, key: "speed_sensor_type", label: "Speed Sensor Type", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 4, default_val: 2, unit: "" },
    ParamDef { addr: 0x42, key: "resolver_poles", label: "Resolver Poles", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 2, max_val: 64, default_val: 2, unit: "" },
    ParamDef { addr: 0x43, key: "min_excitation", label: "Min Excitation", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 255, default_val: 0, unit: "" },
    ParamDef { addr: 0x44, key: "motor_temp_sensor", label: "Motor Temp Sensor", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 3, default_val: 0, unit: "" },
    ParamDef { addr: 0x46, key: "high_temp_cutoff", label: "High Temp Cut off", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 150, default_val: 130, unit: "°C" },
    ParamDef { addr: 0x49, key: "resume_off", label: "Resume off", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 150, default_val: 110, unit: "°C" },
    ParamDef { addr: 0x47, key: "high_temp_striae", label: "High Temp Striae", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 150, default_val: 100, unit: "°C" },
    ParamDef { addr: 0x48, key: "high_temp_week_pct", label: "High Temp Week %", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: false, min_val: 0, max_val: 100, default_val: 0, unit: "%" },
    ParamDef { addr: 0x4D, key: "line_hall_zero", label: "Line Hall Zero", category: OemCategory::Motor, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 1024, default_val: 508, unit: "" },
    ParamDef { addr: 0x4E, key: "line_hall_amp", label: "Line Hall amplitude", category: OemCategory::Motor, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 1024, default_val: 410, unit: "" },
    ParamDef { addr: 0x4F, key: "line_hall_high_err", label: "Line Hall High Err", category: OemCategory::Motor, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 1024, default_val: 972, unit: "" },
    ParamDef { addr: 0x50, key: "line_hall_low_err", label: "Line Hall Low Err", category: OemCategory::Motor, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 1024, default_val: 50, unit: "" },
    ParamDef { addr: 0x51, key: "swap_motor_phase", label: "Swap Motor Phase", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: false, is_critical: true, min_val: 0, max_val: 1, default_val: 0, unit: "" },
    ParamDef { addr: 0x52, key: "resolver_init_angle", label: "Resolver init angle", category: OemCategory::Motor, val_type: ValueType::U16, is_read_only: false, is_critical: true, min_val: 0, max_val: 32767, default_val: 8129, unit: "deg" },

    // --- MOTOR TAB: Read-Only Fields ---
    ParamDef { addr: 0x06, key: "hall_0deg", label: "0° Hall value", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: true, is_critical: false, min_val: 0, max_val: 7, default_val: 5, unit: "" },
    ParamDef { addr: 0x07, key: "hall_60deg", label: "60° Hall value", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: true, is_critical: false, min_val: 0, max_val: 7, default_val: 4, unit: "" },
    ParamDef { addr: 0x0A, key: "hall_120deg", label: "120° Hall value", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: true, is_critical: false, min_val: 0, max_val: 7, default_val: 6, unit: "" },
    ParamDef { addr: 0x0B, key: "hall_180deg", label: "180° Hall value", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: true, is_critical: false, min_val: 0, max_val: 7, default_val: 2, unit: "" },
    ParamDef { addr: 0x0E, key: "hall_240deg", label: "240° Hall value", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: true, is_critical: false, min_val: 0, max_val: 7, default_val: 3, unit: "" },
    ParamDef { addr: 0x0F, key: "hall_300deg", label: "300° Hall value", category: OemCategory::Motor, val_type: ValueType::U8, is_read_only: true, is_critical: false, min_val: 0, max_val: 7, default_val: 1, unit: "" },

    // --- CONTROL TAB ---
    ParamDef { addr: 0x80, key: "iq_kp", label: "IQ Kp", category: OemCategory::Control, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 32767, default_val: 1500, unit: "" },
    ParamDef { addr: 0x82, key: "iq_ki", label: "IQ Ki", category: OemCategory::Control, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 32767, default_val: 30, unit: "" },
    ParamDef { addr: 0x84, key: "id_kp", label: "ID Kp", category: OemCategory::Control, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 32767, default_val: 1500, unit: "" },
    ParamDef { addr: 0x86, key: "id_ki", label: "ID Ki", category: OemCategory::Control, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 32767, default_val: 30, unit: "" },
    ParamDef { addr: 0x8A, key: "ivt_brk_min", label: "IVT BRK Min", category: OemCategory::Control, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 1000, default_val: 50, unit: "" },
    ParamDef { addr: 0x9C, key: "torque_speed_kp", label: "Torque Speed Kp", category: OemCategory::Control, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 32767, default_val: 3000, unit: "" },
    ParamDef { addr: 0x9E, key: "torque_speed_ki", label: "Torque Speed Ki", category: OemCategory::Control, val_type: ValueType::U16, is_read_only: false, is_critical: false, min_val: 0, max_val: 32767, default_val: 80, unit: "" },
];

pub fn export_profile_to_json(profile: &ParamProfile) -> Result<String, String> {
    serde_json::to_string_pretty(profile).map_err(|e| e.to_string())
}

pub fn import_profile_from_json(json_str: &str) -> Result<ParamProfile, String> {
    serde_json::from_str(json_str).map_err(|e| e.to_string())
}
