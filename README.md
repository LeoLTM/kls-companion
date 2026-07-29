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

## 3. Register Memory Map (459 Registers)

### Telemetry & System Monitors (0x00 - 0x13)

| Address (Dec / Hex) | Parameter Name | Size | Type | Range | Description |
|---|---|---|---|---|---|
| `000` (`0x00`) | **Product Model** | 2 byte(s) | `a` | 0..0 | Module Name, Product Model, no range |
| `001` (`0x01`) | **Brake Pedal** | 1 byte(s) | `uo` | 0..255 | Brake AD value, range 0~255 corresponding to 0~5V |
| `002` (`0x02`) | **Brake Switch 1** | 1 byte(s) | `uo` | 0..2 | Brake switch 1 status, range 0 or 1 |
| `003` (`0x03`) | **Safety Switch** | 1 byte(s) | `uo` | 0..2 | Throttle safety switch status, range 0 or 1 |
| `004` (`0x04`) | **Forward Switch** | 1 byte(s) | `uo` | 0..2 | Forward switch status, range 0 or 1 |
| `005` (`0x05`) | **Reverse Switch** | 1 byte(s) | `uo` | 0..2 | Reverse switch status, range 0 or 1 |
| `006` (`0x06`) | **Hall A** | 1 byte(s) | `uo` | 0..2 | Hall A / Encoder A status, range 0 or 1 |
| `007` (`0x07`) | **Hall B** | 1 byte(s) | `uo` | 0..2 | Hall B / Encoder B status, range 0 or 1 |
| `008` (`0x08`) | **Customer Code** | 2 byte(s) | `a` | 0..0 | Special Version，Special Version, no range |
| `009` (`0x09`) | **Battery Voltage** | 1 byte(s) | `uo` | 0..200 | Actual battery voltage, range 0~200V |
| `010` (`0x0A`) | **Motor Temp** | 1 byte(s) | `uo` | 0..150 | Motor temperature, range 0~150°C |
| `011` (`0x0B`) | **Internal Temp** | 1 byte(s) | `uo` | 0..150 | Controller temperature, range 0~150°C |
| `012` (`0x0C`) | **Serial Number** | 2 byte(s) | `h` | 0..0 | Serial Number，Serial Number, no range |
| `013` (`0x0D`) | **Feedback Direction** | 1 byte(s) | `uo` | 0..2 | Actual running direction: 0 Forward, 1 Reverse |
| `014` (`0x0E`) | **Brake Switch 2** | 1 byte(s) | `uo` | 0..255 | Brake switch 2 status, range 0 or 1 |
| `015` (`0x0F`) | **Low Speed Switch** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `016` (`0x10`) | **Software Version** | 2 byte(s) | `h` | 0..0 | Software Version，Software Version, no range |
| `018` (`0x12`) | **Motor Speed** | 2 byte(s) | `uo` | 0..10000 | Motor speed, range 0~10000 |

### Protection & Battery Configuration (0x14 - 0x3F)

| Address (Dec / Hex) | Parameter Name | Size | Type | Range | Description |
|---|---|---|---|---|---|
| `020` (`0x14`) | **Startup High Pedal** | 0 byte(s) | `uo` | 0..1 | 0: Disable, 1: Enable. Startup high pedal protection (prevents runaway on boot if throttle engaged) |
| `021` (`0x15`) | **Throttle Safety Switch** | 0 byte(s) | `uo` | 0..1 | 0: Disable, 1: Enable. Throttle pedal is active when this switch is closed |
| `022` (`0x16`) | **Startup Delay Time** | 1 byte(s) | `uo` | 0..20 | Startup Time，Startup delay time after power-on, range 0~20 |
| `023` (`0x17`) | **Controller Voltage** | 2 byte(s) | `uo` | 0..612 | Controller Voltage，Controller voltage, range 0~612 |
| `024` (`0x18`) | **RC Model Max Value** | 2 byte(s) | `uo` | 0..10000 | RC model remote control max count, range 0~10000 |
| `025` (`0x19`) | **Undervoltage Value** | 2 byte(s) | `uo` | 0..1000 | Low Voltage Value，Undervoltage error threshold, range 0~1000 |
| `026` (`0x1A`) | **Reserved** | 2 byte(s) | `uo` | 0..10000 | Reserved, range 0~10000 |
| `027` (`0x1B`) | **Overvoltage Value** | 2 byte(s) | `uo` | 0..1000 | High Voltage Value，Overvoltage error threshold, range 0~1000 |
| `028` (`0x1C`) | **Reserved** | 2 byte(s) | `uo` | 0..10000 | Reserved, range 0~10000 |
| `029` (`0x1D`) | **Hall Current Sensor Rated Value** | 2 byte(s) | `uo` | 0..1000 | Hall current sensor nominal rating, range 0~1000A |
| `030` (`0x1E`) | **Reserved** | 2 byte(s) | `uo` | 0..10000 | Reserved, range 0~10000 |
| `031` (`0x1F`) | **Phase Max Current AD** | 2 byte(s) | `uo` | 409..2048 | AD value corresponding to rated current of Hall current sensor. Do not modify unless ADC resolution changes. Range 409 (10-bit)~2048 (12-bit) |
| `032` (`0x20`) | **Reserved** | 2 byte(s) | `uo` | 0..10000 | Reserved, range 0~10000 |
| `033` (`0x21`) | **Phase A Zero Current AD** | 2 byte(s) | `uo` | 474..2200 | Phase A Zero current, range 474 (10-bit)~2200 (12-bit) |
| `034` (`0x22`) | **Reserved** | 2 byte(s) | `uo` | 0..10000 | Reserved, range 0~10000 |
| `035` (`0x23`) | **Phase B Zero Current AD** | 2 byte(s) | `uo` | 474..2200 | Phase B Zero current, range 474 (10-bit)~2200 (12-bit) |
| `036` (`0x24`) | **Reserved** | 2 byte(s) | `uo` | 0..10000 | Reserved, range 0~10000 |
| `037` (`0x25`) | **Phase Current Percent** | 1 byte(s) | `uo` | 20..100 | Current Percent, range 20~100 |
| `038` (`0x26`) | **Battery Current Limit** | 1 byte(s) | `uo` | 20..100 | Battry Current Limit，Limits maximum battery current, range 20~100 |
| `039` (`0x27`) | **Battery Current Limit Derating** | 1 byte(s) | `uo` | 20..100 | Battry Current Limit Weaking，Derating starts at Low Voltage Cutoff * 1.15; calibrated value is remaining percentage after derating. Range 20~100 |
| `040` (`0x28`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `041` (`0x29`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `042` (`0x2A`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `043` (`0x2B`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `044` (`0x2C`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `045` (`0x2D`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `046` (`0x2E`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `047` (`0x2F`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `048` (`0x30`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `049` (`0x31`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `050` (`0x32`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `051` (`0x33`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `052` (`0x34`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `053` (`0x35`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `054` (`0x36`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `055` (`0x37`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `056` (`0x38`) | **Motor Parameter Identification Enable** | 1 byte(s) | `uo` | 0..255 | Identify Motor Parameters Enable Flag，0x55 = Exit, 0xAA = Enter |
| `057` (`0x39`) | **PCB Cutoff Temp (°C)** | 1 byte(s) | `uo` | 0..255 | PCB cutoff temperature, nominal 110°C, cuts off on overtemp, range 0~255 |
| `058` (`0x3A`) | **PCB Recovery Temp (°C)** | 1 byte(s) | `uo` | 0..255 | PCB overtemp recovery temperature, nominal 90°C, range 0~255 |
| `059` (`0x3B`) | **PCB Overtemp Start (°C)** | 1 byte(s) | `uo` | 0..255 | PCB overtemp derating start temperature, nominal 100°C, range 0~255 |
| `060` (`0x3C`) | **PCB Overtemp Derating (%)** | 1 byte(s) | `uo` | 0..100 | PCB overtemp derating percentage, nominal 50%. Derating ratio from PCB Overtemp Start (°C) to PCB Cutoff Temp (°C), range 0~100 |
| `061` (`0x3D`) | **PCB Low Temp End (°C)** | 1 byte(s) | `uo` | 0..255 | PCB low temp derating end temperature, nominal 50°C. No derating between this and Mid Temp Start, range 0~255 |
| `062` (`0x3E`) | **PCB Mid Temp Derating (%)** | 1 byte(s) | `uo` | 0..100 | PCB mid temp derating percentage, nominal 35%. Derating ratio from PCB Mid Temp Start (°C) to PCB Overtemp Start (°C), range 0~100 |
| `063` (`0x3F`) | **PCB Low Temp Start (°C)** | 1 byte(s) | `uo` | 0..255 | PCB low temp derating start temperature, nominal 0°C, range 0~255 |

### Speed Limits, Ramps & Braking (0x40 - 0x7F)

| Address (Dec / Hex) | Parameter Name | Size | Type | Range | Description |
|---|---|---|---|---|---|
| `064` (`0x40`) | **PCB Low Temp Derating (%)** | 1 byte(s) | `uo` | 0..100 | PCB low temp derating percentage, nominal 30%. Derating ratio from PCB Low Temp End (°C) to PCB Low Temp Start (°C), range 0~100 |
| `065` (`0x41`) | **PCB Sub-Zero End (°C)** | 1 byte(s) | `so` | 0..255 | PCB sub-zero temp derating end temperature, nominal 216 (-40°C), range 0~255 |
| `066` (`0x42`) | **PCB Sub-Zero Derating (%)** | 1 byte(s) | `uo` | 0..100 | PCB sub-zero temp derating percentage, nominal 40%. Derating ratio from PCB Low Temp Start (°C) to PCB Sub-Zero End (°C), range 0~100 |
| `067` (`0x43`) | **PCB Reference Temp (°C)** | 1 byte(s) | `uo` | 0..255 | PCB reference temperature, nominal 0°C, range 0~255 |
| `068` (`0x44`) | **PCB Mid Temp Start (°C)** | 1 byte(s) | `uo` | 0..255 | PCB mid temp derating start temperature, nominal 80°C, range 0~255 |
| `069` (`0x45`) | **HL Reference Temp (°C)** | 1 byte(s) | `uo` | 0..255 | HL reference temperature, nominal 50°C, range 0~255 |
| `070` (`0x46`) | **HL Overtemp End (°C)** | 1 byte(s) | `uo` | 0..255 | HL overtemp cutoff temperature, nominal 120°C, range 0~255 |
| `071` (`0x47`) | **HL Overtemp Start (°C)** | 1 byte(s) | `uo` | 0..255 | HL overtemp derating start temperature, nominal 90°C, range 0~255 |
| `072` (`0x48`) | **HL Overtemp Derating (%)** | 1 byte(s) | `uo` | 0..100 | HL overtemp derating percentage, nominal 70%. Derating ratio from HL Overtemp Start (°C) to HL Overtemp End (°C), range 0~100 |
| `073` (`0x49`) | **HL Recovery Temp (°C)** | 1 byte(s) | `uo` | 0..255 | HL overtemp recovery temperature, nominal 110°C, range 0~255 |
| `074` (`0x4A`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `075` (`0x4B`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `076` (`0x4C`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `077` (`0x4D`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `078` (`0x4E`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `079` (`0x4F`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `080` (`0x50`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `081` (`0x51`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `082` (`0x52`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `083` (`0x53`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `084` (`0x54`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `085` (`0x55`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `086` (`0x56`) | **Bootloader Mode** | 1 byte(s) | `uo` | 0..255 | Bootloader Mode，Bootloader mode, 0xFF = Enabled, others = Disabled |
| `087` (`0x57`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `088` (`0x58`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `089` (`0x59`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `090` (`0x5A`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `091` (`0x5B`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `092` (`0x5C`) | **Throttle Low Error Point** | 1 byte(s) | `uo` | 0..20 | Reports throttle type error if below calibrated value, range 0~20% |
| `093` (`0x5D`) | **Throttle High Error Point** | 1 byte(s) | `uo` | 80..100 | Reports throttle type error if above calibrated value, range 80~100% |
| `094` (`0x5E`) | **Throttle Ramp Rate** | 1 byte(s) | `uo` | 10..100 | Acc Speed，Throttle response rate, smaller value = faster response, range 10~100 |
| `095` (`0x5F`) | **Throttle Type** | 1 byte(s) | `uo` | 0..3 | TPS Type, Throttle type, range 0: None, 1: 0-5V, 2: 1-4V, 3: 0-5K |
| `096` (`0x60`) | **Throttle Low Deadband** | 1 byte(s) | `uo` | 0..80 | Throttle low deadband, range 0~40 corresponding to 0%~40% |
| `097` (`0x61`) | **Throttle High Deadband** | 1 byte(s) | `uo` | 120..200 | Throttle high deadband, range 60~100 corresponding to 60%~100% |
| `098` (`0x62`) | **Throttle Forward MAP** | 1 byte(s) | `uo` | 0..100 | Throttle forward MAP, range 0~100. Defines % of max throttle at 50% pedal position (curvature) |
| `099` (`0x63`) | **Throttle Reverse MAP** | 1 byte(s) | `uo` | 0..100 | Throttle reverse MAP, range 0~100. Defines % of max reverse throttle at 50% pedal position (curvature) |
| `100` (`0x64`) | **Brake Type** | 1 byte(s) | `uo` | 0..3 | BRAKE Type, Brake type, range 0: None, 1: 0-5V, 2: 1-4V, 3: 0-5K |
| `101` (`0x65`) | **Brake Low Deadband** | 1 byte(s) | `uo` | 0..80 | Brake low deadband, range 0~40 corresponding to 0%~40% |
| `102` (`0x66`) | **Brake High Deadband** | 1 byte(s) | `uo` | 120..200 | Brake high deadband, range 60~100 corresponding to 60%~100% |
| `103` (`0x67`) | **Brake MAP** | 1 byte(s) | `uo` | 0..100 | Brake MAP, range 0~100. Defines % of max brake at 50% pedal position (curvature) |
| `104` (`0x68`) | **Control Mode** | 1 byte(s) | `uo` | 0..2 | Control Mode，0:OpenLoop, 1:Speed CloseLoop,2:Torque CloseLoop |
| `105` (`0x69`) | **Max Output Frequency** | 2 byte(s) | `uo` | 0..300 | Max Output Frequency，Max output frequency, range 0~300Hz |
| `107` (`0x6B`) | **Max Speed** | 2 byte(s) | `uo` | 0..60000 | Max speed，Max motor speed, range 0~60000 RPM |
| `109` (`0x6D`) | **Max Forward Speed %** | 1 byte(s) | `uo` | 30..100 | Forward Speed Limit, range 30~100 |
| `110` (`0x6E`) | **Max Reverse Speed %** | 1 byte(s) | `uo` | 20..100 | Reverse Speed Limit, range 20~100 |
| `111` (`0x6F`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `112` (`0x70`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `113` (`0x71`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `114` (`0x72`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `115` (`0x73`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `116` (`0x74`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `117` (`0x75`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `118` (`0x76`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `119` (`0x77`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `120` (`0x78`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `121` (`0x79`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `122` (`0x7A`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `123` (`0x7B`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `124` (`0x7C`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `125` (`0x7D`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `126` (`0x7E`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `127` (`0x7F`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |

### FOC Motor Calibration & PID Loops (0x80 - 0xAF)

| Address (Dec / Hex) | Parameter Name | Size | Type | Range | Description |
|---|---|---|---|---|---|
| `128` (`0x80`) | **Q-Axis Current Loop Kp** | 2 byte(s) | `uo` | 0..32767 | Q-axis current loop proportional gain Kp, range 0~32767 corresponding to 0~3.9999 |
| `130` (`0x82`) | **Q-Axis Current Loop Ki** | 2 byte(s) | `uo` | 0..32767 | Q-axis current loop integral gain Ki, range 0~32767 corresponding to 0~3.9999 |
| `132` (`0x84`) | **Q-Axis Current Loop Limit** | 2 byte(s) | `uo` | 32767..56756 | Q-axis current loop output limit, range 32767~56756 corresponding to 0.9999~1.732 |
| `134` (`0x86`) | **D-Axis Current Loop Kp** | 2 byte(s) | `uo` | 0..32767 | D-axis current loop proportional gain Kp, range 0~32767 corresponding to 0~3.9999 |
| `136` (`0x88`) | **D-Axis Current Loop Ki** | 2 byte(s) | `uo` | 0..32767 | D-axis current loop integral gain Ki, range 0~32767 corresponding to 0~3.9999 |
| `138` (`0x8A`) | **D-Axis Current Loop Limit** | 2 byte(s) | `uo` | 23170..32767 | D-axis current loop output limit, range 23170~32767 corresponding to 0.707~0.9999 |
| `140` (`0x8C`) | **Voltage Loop Kp** | 2 byte(s) | `uo` | 0..32767 | Voltage loop proportional gain Kp, range 0~32767 corresponding to 0~3.9999 |
| `142` (`0x8E`) | **Voltage Loop Ki** | 2 byte(s) | `uo` | 0..32767 | Voltage loop integral gain Ki, range 0~32767 corresponding to 0~3.9999 |
| `144` (`0x90`) | **Voltage Loop Error Limit** | 2 byte(s) | `uo` | 50..4095 | Voltage loop error signal limit, range 50~4095 |
| `146` (`0x92`) | **Switch Point RPM Upper Limit** | 2 byte(s) | `uo` | 0..65535 | Motor RPM threshold when switching from square wave to sine wave mode with Hall sensor, range 0~65535 RPM |
| `147` (`0x93`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `148` (`0x94`) | **Switch Point RPM Lower Limit** | 2 byte(s) | `uo` | 0..65535 | Motor RPM threshold when switching from sine wave to square wave mode with Hall sensor, range 0~65535 RPM |
| `149` (`0x95`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `150` (`0x96`) | **Iq Compensation Loop Kp** | 2 byte(s) | `uo` | 0..32767 | Torque current Iq compensation loop Kp, range 0~32767 corresponding to 0~3.9999 |
| `151` (`0x97`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `152` (`0x98`) | **Iq Compensation Loop Ki** | 2 byte(s) | `uo` | 0..32767 | Torque current Iq compensation loop Ki, range 0~32767 corresponding to 0~3.9999 |
| `153` (`0x99`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `154` (`0x9A`) | **Iq Compensation Loop Error Limit** | 2 byte(s) | `uo` | 50..4095 | Torque current Iq compensation loop error limit, range 50~4095 |
| `155` (`0x9B`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `156` (`0x9C`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `157` (`0x9D`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `158` (`0x9E`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `159` (`0x9F`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `160` (`0xA0`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `161` (`0xA1`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `162` (`0xA2`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `163` (`0xA3`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `164` (`0xA4`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `165` (`0xA5`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `166` (`0xA6`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `167` (`0xA7`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `168` (`0xA8`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `169` (`0xA9`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `170` (`0xAA`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `171` (`0xAB`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `172` (`0xAC`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `173` (`0xAD`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `174` (`0xAE`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |
| `175` (`0xAF`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved, range 0~255 |

### Advanced & CAN Bus / J1939 Settings (0x1B0 - 0x1FD)

| Address (Dec / Hex) | Parameter Name | Size | Type | Range | Description |
|---|---|---|---|---|---|
| `432` (`0x1B0`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `433` (`0x1B1`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `434` (`0x1B2`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `435` (`0x1B3`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `436` (`0x1B4`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `437` (`0x1B5`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `438` (`0x1B6`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `439` (`0x1B7`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `440` (`0x1B8`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `441` (`0x1B9`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `442` (`0x1BA`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `443` (`0x1BB`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `444` (`0x1BC`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `445` (`0x1BD`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `446` (`0x1BE`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `447` (`0x1BF`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `448` (`0x1C0`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `449` (`0x1C1`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `450` (`0x1C2`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `451` (`0x1C3`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `452` (`0x1C4`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `453` (`0x1C5`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `454` (`0x1C6`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `455` (`0x1C7`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `456` (`0x1C8`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `457` (`0x1C9`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `458` (`0x1CA`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `459` (`0x1CB`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `460` (`0x1CC`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `461` (`0x1CD`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `462` (`0x1CE`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `463` (`0x1CF`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `464` (`0x1D0`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `465` (`0x1D1`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `466` (`0x1D2`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `467` (`0x1D3`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `468` (`0x1D4`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `469` (`0x1D5`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `470` (`0x1D6`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `471` (`0x1D7`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `472` (`0x1D8`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `473` (`0x1D9`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `474` (`0x1DA`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `475` (`0x1DB`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `476` (`0x1DC`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `477` (`0x1DD`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `478` (`0x1DE`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `479` (`0x1DF`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `480` (`0x1E0`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `481` (`0x1E1`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `482` (`0x1E2`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `483` (`0x1E3`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `484` (`0x1E4`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `485` (`0x1E5`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `486` (`0x1E6`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `487` (`0x1E7`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `488` (`0x1E8`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `489` (`0x1E9`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `490` (`0x1EA`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `491` (`0x1EB`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `492` (`0x1EC`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `493` (`0x1ED`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `494` (`0x1EE`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `495` (`0x1EF`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `496` (`0x1F0`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `497` (`0x1F1`) | **Reserved** | 1 byte(s) | `uo` | 0..255 | Reserved |
| `498` (`0x1F2`) | **J1939 Preferred Address** | 1 byte(s) | `uo` | 0..253 | J1939 Preferred Address, preferred address, range 0~253 |
| `499` (`0x1F3`) | **J1939 Arbitrary Address Capable** | 1 byte(s) | `uo` | 0..1 | J1939 Arbitrary Address Capable, arbitrary address capable field, range 0~1 |
| `500` (`0x1F4`) | **J1939 Industry Group** | 1 byte(s) | `uo` | 0..7 | J1939 Industry Group, industry group, range 0~7 |
| `501` (`0x1F5`) | **J1939 Vehicle System Instance** | 1 byte(s) | `uo` | 0..15 | J1939 Vehicle System Instance, vehicle system instance field, range 0~15 |
| `502` (`0x1F6`) | **J1939 Vehicle System** | 1 byte(s) | `so` | 0..127 | J1939 Vehicle System, vehicle system field, range 0~127 |
| `503` (`0x1F7`) | **J1939 Reserved Fields** | 1 byte(s) | `uo` | 0..1 | J1939 Reserved Fields, range 0~1 |
| `504` (`0x1F8`) | **J1939 Function Fields** | 1 byte(s) | `uo` | 0..255 | J1939 Function Fields, function fields, range 0~255 |
| `505` (`0x1F9`) | **J1939 Function Instance** | 1 byte(s) | `uo` | 0..31 | J1939 Function Instance, function instance field, range 0~31 |
| `506` (`0x1FA`) | **J1939 ECU Instance** | 1 byte(s) | `uo` | 0..7 | J1939 ECU Instance, ECU instance field, range 0~7 |
| `507` (`0x1FB`) | **J1939 Manufacturer Code** | 2 byte(s) | `uo` | 0..2047 | J1939 Manufacturer Code Instance, manufacturer code field, range 0~2047 |
| `509` (`0x1FD`) | **J1939 Identity Number** | 2 byte(s) | `uo` | 0..2097151 | J1939 Identity Number, identity number field, range 0~2097151 |


---

## 4. Usage Example (Rust)

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
