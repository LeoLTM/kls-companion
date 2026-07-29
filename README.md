# Kelly KLS Controller Protocol & Companion

Rust library and companion application for interfacing with Kelly Controls KLS / KMC series motor controllers via serial / UART.

Built with Rust, egui, serialport and serde.

---

## 1. Serial Protocol Overview

- **Baud Rate**: 19200 bps (default 8N1)
- **Frame Checksum**: `sum(all preceding bytes) % 256`

### Command Frames

| Command | Tx Frame | Rx Frame Format | Purpose |
|---|---|---|---|
| **Query Packet A** | `[0x3A, 0x00, 0x3A]` | `[0x3A, status, tps, brk, ... 19 bytes]` | Realtime telemetry (switches, temps, battery V) |
| **Query Packet B** | `[0x3B, 0x00, 0x3B]` | `[0x3B, 0x00, ..., rpm_hi, rpm_lo, cur_hi, cur_lo, ... 19 bytes]` | Realtime telemetry (motor RPM, phase current) |
| **Read Parameter** | `[0x1B, 0x01, addr, chk]` | `[0x1B, len, addr, val_bytes..., chk]` | Read register `addr` |
| **Write Parameter**| `[0x42, 0x02, addr, val, chk]` | `[0x42, 0x02, addr, val, chk]` | Write register `addr` with `val` |
| **Angle Identify** | Write `170` (`0xAA`) to `0x55` | - | Triggers auto angle identification on reset |

---

## 2. Telemetry Packet Formats

### Packet A (0x3A) - 19 Bytes

| Byte Index | Field Name | Type | Range / Scaling | Description |
|---|---|---|---|---|
| 0 | Header | `u8` | `0x3A` | Packet A identifier |
| 1 | Fault Status | `u8` | Bitfield | Error code / static variable |
| 2 | TPS Pedal | `u8` | `0..255` (0..5V) | Throttle pedal AD value |
| 3 | BRK Pedal | `u8` | `0..255` (0..5V) | Brake pedal AD value |
| 4 | Brake Switch 1 | `bool` | `0` or `1` | Brake pedal switch status |
| 5 | Foot Switch | `bool` | `0` or `1` | Safety / foot switch status |
| 6 | Forward Switch | `bool` | `0` or `1` | Forward direction switch |
| 7 | Reverse Switch | `bool` | `0` or `1` | Reverse direction switch |
| 8 | Hall A | `bool` | `0` or `1` | Hall sensor A status |
| 9 | Hall B | `bool` | `0` or `1` | Hall sensor B status |
| 10 | Hall C | `bool` | `0` or `1` | Hall sensor C status |
| 11 | Battery Voltage | `u8` | `0..200` V | Actual battery voltage |
| 12 | Motor Temp | `u8` | `0..150` °C | Motor temperature |
| 13 | Controller Temp | `u8` | `0..150` °C | Internal controller temperature |
| 14 | Setting Dir | `bool` | `0`=Fwd, `1`=Rev | Requested direction |
| 15 | Actual Dir | `bool` | `0`=Fwd, `1`=Rev | Actual motor rotation direction |
| 16 | Brake Switch 2 | `bool` | `0` or `1` | Secondary brake switch status |
| 17 | Low Speed Switch | `u8` | `0..255` | Low speed mode switch / reserved |
| 18 | Checksum | `u8` | `sum(b[0..17]) % 256` | Frame checksum |

### Packet B (0x3B) - 19 Bytes

| Byte Index | Field Name | Type | Range / Scaling | Description |
|---|---|---|---|---|
| 0 | Header | `u8` | `0x3B` | Packet B identifier |
| 1..3 | Reserved | `u8` | - | Status / reserved |
| 4..5 | Raw RPM | `u16` BE | `raw / 4` = RPM | Motor RPM |
| 6..7 | Phase Current | `u16` BE | Amps / AD | Motor phase current |
| 8..17 | Internal Diagnostics | `u8` | - | Internal AD / hall angles |
| 18 | Checksum | `u8` | `sum(b[0..17]) % 256` | Frame checksum |

---

## 3. Register Memory Map (98 Canonical OEM Parameters)

The KLS protocol queries parameters using single-byte addresses (`0x00`..`0xFE`) via Read Command `0x1B` and Write Command `0x42`. Below is the complete 98-item catalog matching `src/protocol/oem_params.rs`.

### 🚗 Vehicle Parameters

| Address (Hex) | Key | Parameter Label | Type | Bounds | Unit | Access | Description |
|---|---|---|---|---|---|---|---|
| `0x00` | `module_name` | **Module Name** | `ReadOnlyText` | `Text / Factory` | - | 🔒 Read-Only | Controller hardware model designation (e.g. KLS7230S). Read-only factory identifier. |
| `0x08` | `user_name` | **User Name** | `ReadOnlyText` | `Text / Factory` | - | 🔒 Read-Only | Custom factory customer variant code / firmware identifier. |
| `0x0C` | `serial_number` | **Serial Number** | `ReadOnlyText` | `Text / Factory` | - | 🔒 Read-Only | Unique factory hardware serial number. |
| `0x10` | `sw_version` | **Software Version** | `ReadOnlyText` | `Text / Factory` | - | 🔒 Read-Only | Controller firmware version number. |
| `0x17` | `ctrl_volt` | **Controller Volt** | `U16` | `0..144` | V | 🔒 Read-Only | Nominal battery pack voltage rating (e.g. 72V). Read-only hardware specification. |
| `0x1D` | `hall_galv` | **Hall** | `U16` | `0..1000` | A | 🔒 Read-Only | Internal Hall galvanometer current sensor scale rating in Amperes. |
| `0x1F` | `phase_curr_max_ad` | **PhaseCurr Max AD** | `U16` | `409..2048` | AD | 🔒 Read-Only | Maximum ADC raw sampling value corresponding to peak phase current. |
| `0x84` | `brake_sw_level` | **Brake_SW_Level** | `U8` | `0..255` | - | 🔒 Read-Only | Active hardware level configuration for brake switch inputs. |
| `0x99` | `j_can_address` | **J CAN Address** | `U8` | `0..255` | - | ✏️ Read/Write | J1939 CAN bus node device address (0-255). Default is 127 (or 5 for standard CAN nodes). |
| `0x19` | `low_volt` | **Low Volt** | `U16` | `18..180` | V | ⚠️ Write (Critical) | Minimum battery voltage cut-off (V). If battery voltage drops below this threshold for 5s, power cutback occurs to protect cells from over-discharge. |
| `0x1B` | `over_volt` | **Over Volt** | `U16` | `18..180` | V | ⚠️ Write (Critical) | Maximum battery voltage limit (V). Controller shuts down motor drive and disables regen if battery voltage exceeds this threshold to prevent over-charging or MOSFET failure. |
| `0x25` | `curr_percent` | **Motor_Current%** | `U8` | `20..100` | % | ⚠️ Write (Critical) | Maximum motor phase current limit as a percentage (20-100%) of controller peak current. Directly scales peak acceleration torque. |
| `0x26` | `battry_limit` | **Battery_Current%** | `U8` | `20..100` | % | ⚠️ Write (Critical) | Maximum battery current limit as a percentage (20-100%) of maximum battery current. Protects battery BMS and limits total DC power draw from battery. |
| `0x38` | `id_angle` | **Identify Angle** | `U8` | `85..170` | deg | ⚠️ Write (Critical) | Motor auto-identification command status. Set to 170 to initiate self-identification on next power cycle (motor spins automatically to measure Hall angles). Automatically resets to 85 when complete. |
| `0x5C` | `tps_low_err` | **TPS Low Err** | `U8` | `0..20` | % | ✏️ Read/Write | Throttle lower error threshold (0-20%). Triggers TPS error code if throttle voltage falls below this percentage (20% = 1.0V). |
| `0x5D` | `tps_high_err` | **TPS High Err** | `U8` | `80..100` | % | ✏️ Read/Write | Throttle upper error threshold (80-100%). Triggers TPS error code if throttle voltage exceeds this percentage (95% = 4.75V) to detect shorted signals. |
| `0x5F` | `tps_type` | **TPS Type** | `U8` | `1..2` | - | ✏️ Read/Write | Throttle sensor type selection. 1: 0-5K 3-wire potentiometer pedal; 2: 0-5V Hall active pedal/twist-grip. |
| `0x60` | `tps_dead_low` | **TPS Dead Low** | `U8` | `0..60` | % | ✏️ Read/Write | Throttle start deadzone lower limit (0-60%). Throttle output stays at 0% until pedal is pressed past this percentage (20% = 1.0V). |
| `0x61` | `tps_dead_high` | **TPS Dead High** | `U8` | `60..95` | % | ✏️ Read/Write | Throttle full-power deadzone upper limit (60-95%). Throttle reaches 100% full output when pedal reaches this percentage (80% = 4.0V). |
| `0x62` | `tps_fwd_map` | **TPS Fwd MAP** | `U8` | `0..100` | % | ✏️ Read/Write | Forward throttle response curve midpoint (0-100%). Adjusts throttle sensitivity curve shape at 50% pedal position (higher value = punchier initial response). |
| `0x63` | `tps_rev_map` | **TPS Rev MAP** | `U8` | `0..100` | % | ✏️ Read/Write | Reverse throttle response curve midpoint (0-100%). Adjusts reverse throttle curve shape at 50% pedal position. |
| `0x64` | `brake_type` | **Brake Type** | `U8` | `0..2` | - | ✏️ Read/Write | Regenerative braking input mode. 0: Switch regen (digital switch input); 1: 0-5K resistance pedal analog regen; 2: 0-5V active Hall pedal analog regen. |
| `0x65` | `brake_dead_low` | **Brake Dead Low** | `U8` | `5..40` | % | ✏️ Read/Write | Analog brake pedal lower deadzone limit (5-40%). Regen braking starts when pedal exceeds this percentage (20% = 1.0V). |
| `0x66` | `brake_dead_high` | **Brake Dead High** | `U8` | `60..95` | % | ✏️ Read/Write | Analog brake pedal upper deadzone limit (60-95%). Maximum regen torque is reached when pedal reaches this percentage (80% = 4.0V). |
| `0x69` | `max_output_fre` | **Max Output Fre** | `U16` | `50..1200` | Hz | ✏️ Read/Write | Maximum electrical fundamental output frequency (50-1200 Hz). Limits maximum achievable motor electrical frequency. |
| `0x6B` | `max_speed` | **Max Speed** | `U16` | `0..16000` | RPM | ✏️ Read/Write | Motor maximum mechanical speed limit in RPM (0-16000 RPM). Limits top vehicle speed. |
| `0x6D` | `max_fwd_speed_pct` | **Max Fwd Speed %** | `U8` | `0..100` | % | ✏️ Read/Write | Maximum forward speed limit as a percentage (0-100%) of Motor Max Speed. |
| `0x6E` | `max_rev_speed_pct` | **Max Rev Speed %** | `U8` | `0..100` | % | ✏️ Read/Write | Maximum reverse speed limit as a percentage (0-100%) of Motor Max Speed. |
| `0x70` | `midspeed_forw` | **MidSpeed Forw** | `U8` | `0..100` | % | ✏️ Read/Write | Maximum forward speed in middle speed gear (0-100%) when 3-speed switch is enabled. |
| `0x71` | `midspeed_rev` | **MidSpeed Rev** | `U8` | `0..100` | % | ✏️ Read/Write | Maximum reverse speed in middle speed gear (0-100%) when 3-speed switch is enabled. |
| `0x72` | `lowspeed_forw` | **LowSpeed Forw** | `U8` | `0..100` | % | ✏️ Read/Write | Maximum forward speed in low speed gear (0-100%) when 3-speed switch is enabled. |
| `0x73` | `lowspeed_rev` | **LowSpeed Rev** | `U8` | `0..100` | % | ✏️ Read/Write | Maximum reverse speed in low speed gear (0-100%) when 3-speed switch is enabled. |
| `0x74` | `three_speed` | **Three Speed** | `U8` | `0..2` | - | ✏️ Read/Write | Number of gear speed modes enabled. 0: 1-speed (max speed mode only); 1: 2-speed mode (mid & max); 2: 3-speed mode (low, mid, max). |
| `0x75` | `pwm_frequency` | **PWM frequency** | `U8` | `10..20` | kHz | ✏️ Read/Write | SVPWM carrier modulation frequency (10, 16, or 20 kHz). 20 kHz provides silent motor operation; lower frequencies reduce MOSFET switching heat. |

### ⚡ Motor Parameters

| Address (Hex) | Key | Parameter Label | Type | Bounds | Unit | Access | Description |
|---|---|---|---|---|---|---|---|
| `0x20` | `motor_nom_curr` | **Motor Nominal** | `U16` | `0..1000` | A | ⚠️ Write (Critical) | Motor nominal current setting during auto-identification (0-1000 A). Set to match motor rated current for proper parameter identification. |
| `0x40` | `motor_poles` | **Motor Poles** | `U8` | `2..128` | - | ⚠️ Write (Critical) | Number of motor magnetic poles (2-128 = 2x pole pairs). Crucial for accurate RPM calculation and electrical commutation. |
| `0x41` | `speed_sensor_type` | **Speed Sensor** | `U8` | `2..3` | - | ✏️ Read/Write | Primary motor position sensor type. 2: 3-phase Hall effect sensors; 3: Magnetic encoder / resolver. |
| `0x42` | `resolver_poles` | **Resolver Poles** | `U8` | `2..32` | - | ✏️ Read/Write | Number of poles for resolver sensor (2-32). Reserved for resolver equipped motors. |
| `0x43` | `min_excitation` | **Min Excitation** | `U16` | `0..100` | - | ✏️ Read/Write | Minimum excitation current coefficient (0-100 A) for field weakening. If 0, field weakening speed boost is inactive. |
| `0x44` | `motor_temp_sensor` | **Motor Temp** | `U8` | `0..2` | - | ✏️ Read/Write | Motor temperature thermistor type. 0: Disabled / None; 1: KTY84-130 / KTY84-150; 2: KTY83-122. |
| `0x46` | `high_temp_cutoff` | **High Temp Cut ℃** | `U8` | `60..170` | °C | ✏️ Read/Write | Motor over-temperature shutdown threshold (60-170 °C). Controller stops driving if motor temperature exceeds this value. |
| `0x47` | `high_temp_striae` | **High Temp Str℃** | `U8` | `0..170` | °C | ✏️ Read/Write | Motor high-temperature current foldback start threshold (0-170 °C). Drive current begins ramping down above this temp. |
| `0x48` | `high_temp_week_pct` | **High Temp Week %** | `U8` | `0..100` | % | ✏️ Read/Write | Motor high-temperature foldback strength percentage (0-100%). Defines current reduction percentage between start and cutoff temp. |
| `0x49` | `resume_off` | **Resume ℃** | `U8` | `60..170` | °C | ✏️ Read/Write | Motor over-temperature recovery threshold (60-170 °C). Controller resumes operation once motor cools down to this temp. |
| `0x4D` | `line_hall_zero` | **Line Hall Zero** | `U16` | `1..1023` | - | ✏️ Read/Write | Sine/Cosine sensor zero-point voltage calibration value (1-1023). Formula: Zero V = Value / 1024 * 5V. |
| `0x4E` | `line_hall_amp` | **Line Hall** | `U16` | `1..1024` | - | ✏️ Read/Write | Sine/Cosine sensor signal amplitude calibration value (1-1024). Valid range 153.6 to 256 for normal signal voltage. |
| `0x4F` | `line_hall_high_err` | **Line Hall High** | `U16` | `1..1023` | - | ✏️ Read/Write | Sine/Cosine sensor high-amplitude error threshold limit (1-1023). Triggers angle sensor fault if amplitude exceeds this. |
| `0x50` | `line_hall_low_err` | **Line Hall Low** | `U16` | `1..1023` | - | ✏️ Read/Write | Sine/Cosine sensor low-amplitude error threshold limit (1-1023). Triggers angle sensor fault if amplitude drops below this. |
| `0x51` | `swap_motor_phase` | **Swap Motor Phase** | `U8` | `0..255` | - | 🔒 Read-Only | Motor phase swapping status flag for sine/cosine sensor alignment. 0: Disabled; 1: Enabled; 255: Identification error. |
| `0x52` | `resolver_init_angle` | **Resolver init** | `U16` | `0..65535` | deg | 🔒 Read-Only | Synchro initial angle reference point (0-65535) for sine/cosine or magnetic encoder position alignment. |
| `0x06` | `hall_0deg` | **0° Hall value** | `U8` | `0..7` | - | 🔒 Read-Only | Hall sensor sequence value at 0° electrical angle. Auto-generated during self-identification. |
| `0x07` | `hall_60deg` | **60° Hall value** | `U8` | `0..7` | - | 🔒 Read-Only | Hall sensor sequence value at 60° electrical angle. Auto-generated during self-identification. |
| `0x0A` | `hall_120deg` | **120° Hall value** | `U8` | `0..7` | - | 🔒 Read-Only | Hall sensor sequence value at 120° electrical angle. Auto-generated during self-identification. |
| `0x0B` | `hall_180deg` | **180° Hall value** | `U8` | `0..7` | - | 🔒 Read-Only | Hall sensor sequence value at 180° electrical angle. Auto-generated during self-identification. |
| `0x0E` | `hall_240deg` | **240° Hall value** | `U8` | `0..7` | - | 🔒 Read-Only | Hall sensor sequence value at 240° electrical angle. Auto-generated during self-identification. |
| `0x0F` | `hall_300deg` | **300° Hall value** | `U8` | `0..7` | - | 🔒 Read-Only | Hall sensor sequence value at 300° electrical angle. Auto-generated during self-identification. |
| `0x53` | `fwd_ha_rising` | **Forward HA (Rising)** | `U8` | `0..7` | - | 🔒 Read-Only | Forward direction Hall-A rising edge sequence value. Auto-generated during self-identification. |
| `0x54` | `fwd_ha_falling` | **Forward HA (Falling)** | `U8` | `0..7` | - | 🔒 Read-Only | Forward direction Hall-A falling edge sequence value. Auto-generated during self-identification. |
| `0x55` | `rev_ha_rising` | **Reverse HA (Rising)** | `U8` | `0..7` | - | 🔒 Read-Only | Reverse direction Hall-A rising edge sequence value. Auto-generated during self-identification. |
| `0x57` | `rev_ha_falling` | **Reverse HA (Falling)** | `U8` | `0..7` | - | 🔒 Read-Only | Reverse direction Hall-A falling edge sequence value. Auto-generated during self-identification. |

### 🎛 Control Parameters

| Address (Hex) | Key | Parameter Label | Type | Bounds | Unit | Access | Description |
|---|---|---|---|---|---|---|---|
| `0x80` | `iq_kp` | **IQ Kp** | `U16` | `0..32767` | - | ✏️ Read/Write | Proportional gain (Kp) for Q-axis (torque) current loop at low speed (<400 RPM). Higher values speed up torque response but may cause startup vibration. |
| `0x82` | `iq_ki` | **IQ Ki** | `U16` | `0..32767` | - | ✏️ Read/Write | Integral gain (Ki) for Q-axis (torque) current loop at low speed (<400 RPM). Improves steady-state current accuracy. |
| `0x86` | `id_kp` | **ID Kp** | `U16` | `0..32767` | - | ✏️ Read/Write | Proportional gain (Kp) for D-axis (flux) current loop at low speed (<400 RPM). Controls field weakening and d-axis current response. |
| `0x88` | `id_ki` | **ID Ki** | `U16` | `0..32767` | - | ✏️ Read/Write | Integral gain (Ki) for D-axis (flux) current loop at low speed (<400 RPM). Enhances flux control stability. |
| `0x96` | `ms_acqr_kp` | **MS_ACQR_Kp** | `U16` | `0..32767` | - | ✏️ Read/Write | Proportional gain (Kp) for Q-axis current loop at medium/high speed (>400 RPM). Accelerates high-speed acceleration response. |
| `0x98` | `ms_acqr_ki` | **MS_ACQR_Ki** | `U16` | `0..32767` | - | ✏️ Read/Write | Integral gain (Ki) for Q-axis current loop at medium/high speed (>400 RPM). Maintains torque precision at higher RPM. |
| `0x9A` | `ms_acdr_kp` | **MS_ACDR_Kp** | `U16` | `0..32767` | - | ✏️ Read/Write | Proportional gain (Kp) for D-axis current loop at medium/high speed (>400 RPM). Optimizes high-speed field weakening response. |
| `0x9C` | `ms_acdr_ki` | **MS_ACDR_Ki** | `U16` | `0..32767` | - | ✏️ Read/Write | Integral gain (Ki) for D-axis current loop at medium/high speed (>400 RPM). Ensures stability during high-speed field weakening. |
| `0x9D` | `brk_ad_brk_pct` | **BRK_AD Brk %** | `U8` | `0..50` | % | ✏️ Read/Write | Analog brake pedal regen strength (0-50%). Sets maximum braking torque percentage when using analog brake pedal regen. |
| `0x9E` | `anti_theft_pct` | **Anti-theft %** | `U8` | `0..30` | % | ✏️ Read/Write | Anti-theft locking torque strength (0-30%). Percentage of maximum current used to lock motor rotor when alarm is triggered. |
| `0x9F` | `brk_speed_limit` | **Brk_Speed Limit** | `U16` | `0..500` | RPM | ✏️ Read/Write | Minimum motor speed threshold (0-500 RPM) to activate regen braking. Regen automatically disengages below this RPM. |
| `0xA0` | `rls_tps_brk_pct` | **RLS_TPS Brk %** | `U8` | `0..50` | % | ✏️ Read/Write | Throttle release (off-throttle / overrun) regen braking strength (0-50%). Provides engine-braking effect when releasing throttle. |
| `0xA1` | `ntl_brk_pct` | **NTL Brk %** | `U8` | `0..50` | % | ✏️ Read/Write | Neutral gear regen braking strength (0-50%). Sets braking torque percentage when vehicle is rolling in neutral. |
| `0xA2` | `accel_time` | **Accel Time** | `U8` | `1..250` | x0.1s | ✏️ Read/Write | Drive torque ramp-up time (1-250 = 0.1s to 25.0s). Time taken for torque to rise from 0 to maximum. Lower value = faster throttle acceleration. |
| `0xA3` | `accel_rls_time` | **Accel Rls Time** | `U8` | `1..250` | x0.1s | ✏️ Read/Write | Drive torque release delay time (1-250 = 0.1s to 25.0s). Time taken for drive torque to ramp down from maximum to 0 upon releasing throttle. |
| `0xA4` | `brake_time` | **Brake Time** | `U8` | `1..250` | x0.1s | ✏️ Read/Write | Braking torque ramp-up time (1-250 = 0.1s to 25.0s). Time taken for regen braking torque to ramp up from 0 to maximum. Provides smooth braking onset. |
| `0xA5` | `brake_rls_time` | **Brake Rls Time** | `U8` | `1..250` | x0.1s | ✏️ Read/Write | Braking torque release delay time (1-250 = 0.1s to 25.0s). Time taken for regen braking torque to decay from maximum to 0. |
| `0xA6` | `brk_sw_brk_pct` | **BRK_SW Brk %** | `U8` | `0..50` | % | ✏️ Read/Write | Digital brake switch regen braking strength (0-50%). Braking torque percentage when brake switch input is activated. |
| `0xA7` | `change_dir_brk_pct` | **Change Dir Brk%** | `U8` | `0..50` | % | ✏️ Read/Write | Direction change (plugging) regen braking strength (0-50%). Braking torque applied when switching FWD/REV while moving. |
| `0xA8` | `compensation` | **Compensation** | `U8` | `0..100` | % | ✏️ Read/Write | Hill-hold / anti-slip compensation torque percentage (0-100%). Provides hold torque to prevent rolling backward on inclines. |
| `0xA9` | `ivt_brk_max` | **IVT BRK Max** | `U16` | `0..10000` | RPM | ✏️ Read/Write | Maximum motor RPM limit (0-10000 RPM) allowing direction change plugging regen braking. |
| `0xAA` | `ivt_brk_min` | **IVT BRK Min** | `U16` | `0..5000` | RPM | ✏️ Read/Write | Minimum motor RPM limit (0-5000 RPM) required to engage direction change plugging regen braking. |
| `0xFA` | `torque_speed_kp` | **Torque Speed Kp** | `U16` | `0..10000` | - | ✏️ Read/Write | Proportional gain (Kp) for Q-axis loop in Torque Mode at low speed (<400 RPM). Tunes startup responsiveness. |
| `0xFC` | `torque_speed_ki` | **Torque Speed Ki** | `U16` | `0..500` | - | ✏️ Read/Write | Integral gain (Ki) for Q-axis loop in Torque Mode at low speed (<400 RPM). Tunes steady-state torque accuracy. |
| `0xFE` | `speed_err_limit` | **Speed Err Limit** | `U16` | `50..4000` | - | ✏️ Read/Write | Speed loop error signal limit (50-4000) for torque mode control loops. |

### ☑ Feature Flags & Checkboxes

| Address (Hex) | Key | Parameter Label | Type | Bounds | Unit | Access | Description |
|---|---|---|---|---|---|---|---|
| `0x14` | `startup_h_pedal` | **Startup H-Pedal** | `Bool` | `0 or 1` | - | ✏️ Read/Write | High-Pedal Protection on Startup (HPD). When enabled, prevents accidental runaway if throttle is pressed when powering on controller. |
| `0x21` | `brake_h_pedal` | **Brake H-Pedal** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Brake High-Pedal Protection. When enabled, reports a high-pedal fault and cuts motor output if throttle and brake are pressed simultaneously. |
| `0x22` | `ntl_h_pedal` | **NTL H-Pedal** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Neutral High-Pedal Protection. When enabled, prevents accidental starting if throttle is pressed while shifting gears out of neutral. |
| `0x23` | `joystick` | **Joystick** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Joystick Throttle Mode. Enables bi-directional 0-5V joystick (0-2.5V = Reverse, 2.5V = Neutral, 2.5V-5V = Forward). |
| `0x24` | `three_gears_switch` | **Three Gears Switch** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Three-Gear Operating Switch. Enabled: FWD / Neutral / REV 3-position gear selection; Disabled: Forward only. |
| `0x27` | `boost` | **Boost** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Boost Switch Mode. Enabled: Connecting Pin 2 (Brake_AN) to 12V triggers full boost power regardless of throttle position; Disabled: Pin 2 operates variable regen. |
| `0x28` | `foot_switch` | **Foot Switch** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Throttle Enable Microswitch (Foot_SW). Enabled: Pin 15 must be connected to 12V for throttle output to be active. |
| `0x29` | `sw_level` | **SW Level** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Digital Switch Input Voltage Level Logic. Enabled (Checked): High active (12V = ON); Disabled (Unchecked): Low active (0V / GND = ON). |
| `0x2A` | `controller_type_kim` | **0,KIM;1,KIM** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Controller internal variant selection bit. Checked: KIM model family logic; Unchecked: HIM model family logic. |
| `0x2B` | `cruise` | **Cruise** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Cruise Control Function. Enabled: Holding steady throttle for >3s enters cruise mode. Automatically disengages if eRPM drops below 4000 or brake is hit. |
| `0x2C` | `anti_theft_en` | **Anti-theft** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Anti-Theft Alarm Lock Function. Enabled: Controller resists motor rotation and applies counter-torque when external anti-theft alarm is triggered. |
| `0x2D` | `anti_slip` | **Anti-Slip** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Bidirectional Anti-Slip Hill Hold. Enabled: Controller detects backward rollback from standstill and applies braking/hold torque to prevent vehicle from slipping. |
| `0x2E` | `change_direction` | **Change Direction** | `Bool` | `0 or 1` | - | ✏️ Read/Write | Motor Rotation Direction Swap. Enabled: Reverses motor direction after auto-identification without swapping physical motor phase wires. |

---

## 4. GUI Companion Application & Interactive Visualizers

The `kls-companion` desktop application provides a graphical interface built with `eframe`/`egui` for real-time telemetry monitoring and complete OEM controller configuration.

### Key Features
- **Real-Time Telemetry Dashboard & Live Charts**: High-frequency polling of battery voltage, phase current, motor RPM, controller temperature, motor temperature, direction, and switch states using `egui_plot`.
- **Full OEM Parameter Catalog (98 Parameters)**: Complete implementation of all 98 items from the official Kelly KLS User Manual & Motor ETS software across four sub-tabs:
  - **🚗 Vehicle**: Voltage cut-offs (`Low Volt`, `Over Volt`), current limits (`Motor_Current%`, `Battery_Current%`), throttle & brake deadzones/MAP curves, speed limits, speed mode switches.
  - **⚡ Motor**: Pole pairs, speed sensor types (Hall vs Encoder/Resolver), temperature foldback thresholds (`High Temp Cut`, `Resume ℃`, `High Temp Str`), Line Hall calibration, and auto-ID Hall angle sequences.
  - **🎛 Control**: FOC current loop gains (`IQ Kp/Ki`, `ID Kp/Ki`), high-speed gains (`MS_ACQR_Kp/Ki`, `MS_ACDR_Kp/Ki`), acceleration/braking ramp times, regen strength percentages, and torque-mode speed loop limits.
  - **☑ Features**: Boolean toggle switches for all 13 feature checkboxes (`Startup H-Pedal`, `Brake H-Pedal`, `NTL H-Pedal`, `Joystick`, `Three Gears Switch`, `Boost`, `Foot Switch`, `SW Level`, `0,KIM;1,KIM`, `Cruise`, `Anti-theft`, `Anti-Slip`, `Change Direction`).
- **Rich Hover Tooltips**: Hovering over any parameter label, address, value slider, checkbox, or write button displays a detailed explanation of its function, formulas, and tuning behavior synthesized from official manuals and firmware definitions.
- **Interactive Dual-Pane Visualizer Panel**: Live graphical curve rendering embedded directly beside parameter controls:
  - **Throttle & Brake MAP Curves**: Visualizes non-linear response curves (0-100%) based on deadzones and midpoint MAP curvature.
  - **PI Step Response Simulation**: Interactive step response plot showing proportional and integral loop gains (`Kp`/`Ki`).
  - **RPM Mode Transition Plot**: Visualizes square-wave to sine-wave mode switching thresholds.
  - **Ramp Dynamics Graph**: Renders drive and braking torque acceleration/release time profiles.
  - **Thermal Foldback Curve**: Graphs motor temperature derating thresholds (`Str ℃` to `Cut ℃`).
  - **3-Phase Hall Electrical Angle Diagram**: Circular electrical degree diagram displaying Hall sensor sequence values (0°, 60°, 120°, 180°, 240°, 300°).
- **Safety Write Protection & Confirmation Modals**: Automatic write protection when motor is active (`RPM > 0`) and warning confirmation popups for critical parameters (voltage thresholds, identification angle, pole count, current limits).
- **JSON Profile Export & Import**: Save and restore full controller configuration profiles to JSON files.

---

## 5. Usage Example (Rust)

```rust
use kls_companion::protocol::kls::{KlsCommand, parse_packet_a, parse_packet_b};

fn main() {
    // Build Packet A query frame
    let cmd_frame = KlsCommand::QueryPacketA.build_frame();
    assert_eq!(cmd_frame, vec![0x3A, 0x00, 0x3A]);

    // Parse Packet A response
    let raw_rx: Vec<u8> = vec![
        0x3A, 0x00, 0x80, 0x00, 0x00, 0x01, 0x01, 0x00,
        0x01, 0x00, 0x01, 0x48, 0x23, 0x2A, 0x00, 0x00,
        0x00, 0x00, 0x57
    ];
    let pkt = parse_packet_a(&raw_rx).unwrap();
    println!("Battery Voltage: {} V", pkt.battery_voltage);
    println!("Motor Temp: {} C", pkt.motor_temp);
}
```
