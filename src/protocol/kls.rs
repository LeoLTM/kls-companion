// Kelly KLS7230 Protocol Decoder
// Implements 19-byte Packet A (0x3A) and Packet B (0x3B) decoding.

pub const PACKET_LENGTH: usize = 19;
pub const CRC_INDEX: usize = 18;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PacketA {
    pub command: u8,
    pub static_var: u8,
    pub throttle: u8,
    pub brake_pedal: u8,
    pub brake_switch: bool,
    pub foot_switch: bool,
    pub forward_switch: bool,
    pub reverse: bool,
    pub hall_a: bool,
    pub hall_b: bool,
    pub hall_c: bool,
    pub battery_voltage: u8,
    pub motor_temp: u8,
    pub controller_temp: u8,
    pub setting_dir: bool,
    pub actual_dir: bool,
    pub brake_switch_2: bool,
    pub low_speed_switch: u8,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PacketB {
    pub rpm: u16,
    pub phase_current: u16,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KlsTelemetry {
    // Packet A telemetry
    pub throttle: u8,
    pub brake_pedal: u8,
    pub brake_switch: bool,
    pub foot_switch: bool,
    pub forward_switch: bool,
    pub reverse: bool,
    pub hall_a: bool,
    pub hall_b: bool,
    pub hall_c: bool,
    pub battery_voltage_v: f32,
    pub motor_temp_c: u8,
    pub controller_temp_c: u8,
    pub setting_dir: bool,
    pub actual_dir: bool,

    // Packet B telemetry
    pub rpm: u16,
    pub phase_current_a: f32,

    // Calculated / convenience fields
    pub throttle_pct: u8,
    pub brake_pct: u8,
    pub error_code: u16,
    pub raw_bytes: Vec<u8>,
}

impl KlsTelemetry {
    pub fn update_from_packet_a(&mut self, pkt: &PacketA) {
        self.throttle = pkt.throttle;
        self.brake_pedal = pkt.brake_pedal;
        self.throttle_pct = ((pkt.throttle as u32 * 100) / 255) as u8;
        self.brake_pct = ((pkt.brake_pedal as u32 * 100) / 255) as u8;
        self.brake_switch = pkt.brake_switch;
        self.foot_switch = pkt.foot_switch;
        self.forward_switch = pkt.forward_switch;
        self.reverse = pkt.reverse;
        self.hall_a = pkt.hall_a;
        self.hall_b = pkt.hall_b;
        self.hall_c = pkt.hall_c;
        self.battery_voltage_v = pkt.battery_voltage as f32;
        self.motor_temp_c = pkt.motor_temp;
        self.controller_temp_c = pkt.controller_temp;
        self.setting_dir = pkt.setting_dir;
        self.actual_dir = pkt.actual_dir;
    }

    pub fn update_from_packet_b(&mut self, pkt: &PacketB) {
        self.rpm = pkt.rpm;
        self.phase_current_a = pkt.phase_current as f32;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KlsCommand {
    QueryPacketA,
    QueryPacketB,
    ReadParam { addr: u8 },
    WriteParam { addr: u8, value: u8 },
}

impl KlsCommand {
    pub fn build_frame(&self) -> Vec<u8> {
        match self {
            KlsCommand::QueryPacketA => {
                vec![0x3A, 0x00, 0x3A]
            }
            KlsCommand::QueryPacketB => {
                vec![0x3B, 0x00, 0x3B]
            }
            KlsCommand::ReadParam { addr } => {
                let cmd = 0x1B;
                let len = 0x01;
                let checksum = calculate_frame_checksum(cmd, len, &[*addr]);
                vec![cmd, len, *addr, checksum]
            }
            KlsCommand::WriteParam { addr, value } => {
                let cmd = 0x42;
                let len = 0x02;
                let checksum = calculate_frame_checksum(cmd, len, &[*addr, *value]);
                vec![cmd, len, *addr, *value, checksum]
            }
        }
    }
}

pub fn calculate_frame_checksum(cmd: u8, len: u8, payload: &[u8]) -> u8 {
    let mut sum: u8 = cmd.wrapping_add(len);
    for &b in payload {
        sum = sum.wrapping_add(b);
    }
    sum
}

pub fn calculate_packet_checksum(bytes: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    let limit = std::cmp::min(bytes.len(), CRC_INDEX);
    for &b in &bytes[..limit] {
        sum = sum.wrapping_add(b);
    }
    sum
}

pub fn validate_packet_checksum(bytes: &[u8]) -> bool {
    if bytes.len() < PACKET_LENGTH {
        return false;
    }
    calculate_packet_checksum(bytes) == bytes[CRC_INDEX]
}

pub fn parse_packet_a(bytes: &[u8]) -> Result<PacketA, String> {
    if bytes.len() < PACKET_LENGTH {
        return Err(format!(
            "Packet A frame too short: got {} bytes, expected {}",
            bytes.len(),
            PACKET_LENGTH
        ));
    }
    if bytes[0] != 0x3A {
        return Err(format!("Invalid Packet A header: 0x{:02X}, expected 0x3A", bytes[0]));
    }
    if !validate_packet_checksum(bytes) {
        return Err(format!(
            "Packet A checksum mismatch: calculated 0x{:02X}, got 0x{:02X}",
            calculate_packet_checksum(bytes),
            bytes[CRC_INDEX]
        ));
    }

    Ok(PacketA {
        command: bytes[0],
        static_var: bytes[1],
        throttle: bytes[2],
        brake_pedal: bytes[3],
        brake_switch: bytes[4] != 0,
        foot_switch: bytes[5] != 0,
        forward_switch: bytes[6] != 0,
        reverse: bytes[7] != 0,
        hall_a: bytes[8] != 0,
        hall_b: bytes[9] != 0,
        hall_c: bytes[10] != 0,
        battery_voltage: bytes[11],
        motor_temp: bytes[12],
        controller_temp: bytes[13],
        setting_dir: bytes[14] != 0,
        actual_dir: bytes[15] != 0,
        brake_switch_2: bytes[16] != 0,
        low_speed_switch: bytes[17],
    })
}

pub fn parse_packet_b(bytes: &[u8]) -> Result<PacketB, String> {
    if bytes.len() < PACKET_LENGTH {
        return Err(format!(
            "Packet B frame too short: got {} bytes, expected {}",
            bytes.len(),
            PACKET_LENGTH
        ));
    }
    if bytes[0] != 0x3B {
        return Err(format!("Invalid Packet B header: 0x{:02X}, expected 0x3B", bytes[0]));
    }
    if !validate_packet_checksum(bytes) {
        return Err(format!(
            "Packet B checksum mismatch: calculated 0x{:02X}, got 0x{:02X}",
            calculate_packet_checksum(bytes),
            bytes[CRC_INDEX]
        ));
    }

    let raw_rpm = u16::from_be_bytes([bytes[4], bytes[5]]);
    let rpm = raw_rpm / 4;
    let phase_current = u16::from_be_bytes([bytes[6], bytes[7]]);

    Ok(PacketB { rpm, phase_current })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_frame_checksum() {
        let frame_a = KlsCommand::QueryPacketA.build_frame();
        assert_eq!(frame_a, vec![0x3A, 0x00, 0x3A]);

        let frame_b = KlsCommand::QueryPacketB.build_frame();
        assert_eq!(frame_b, vec![0x3B, 0x00, 0x3B]);
    }

    #[test]
    fn test_parse_valid_packet_a() {
        let mut bytes = vec![0u8; 19];
        bytes[0] = 0x3A;
        bytes[1] = 0x00;
        bytes[2] = 128; // throttle
        bytes[3] = 0;   // brake
        bytes[4] = 0;   // brake sw
        bytes[5] = 1;   // foot sw
        bytes[6] = 1;   // fwd sw
        bytes[7] = 0;   // rev
        bytes[8] = 1;   // hall a
        bytes[9] = 0;   // hall b
        bytes[10] = 1;  // hall c
        bytes[11] = 72; // battery_voltage 72V
        bytes[12] = 35; // motor temp 35C
        bytes[13] = 42; // ctrl temp 42C
        bytes[14] = 0;  // set dir
        bytes[15] = 0;  // act dir
        bytes[16] = 0;
        bytes[17] = 0;

        let crc = calculate_packet_checksum(&bytes);
        bytes[18] = crc;

        let parsed = parse_packet_a(&bytes).unwrap();
        assert_eq!(parsed.throttle, 128);
        assert_eq!(parsed.battery_voltage, 72);
        assert_eq!(parsed.controller_temp, 42);
        assert_eq!(parsed.motor_temp, 35);
        assert!(parsed.foot_switch);
        assert!(parsed.forward_switch);
        assert!(parsed.hall_a);
        assert!(!parsed.hall_b);
        assert!(parsed.hall_c);
    }

    #[test]
    fn test_parse_valid_packet_b() {
        let mut bytes = vec![0u8; 19];
        bytes[0] = 0x3B;
        // RPM = 4000 -> raw_rpm = 16000 (0x3E80)
        bytes[4] = 0x3E;
        bytes[5] = 0x80;
        // Phase current = 150 (0x0096)
        bytes[6] = 0x00;
        bytes[7] = 0x96;

        let crc = calculate_packet_checksum(&bytes);
        bytes[18] = crc;

        let parsed = parse_packet_b(&bytes).unwrap();
        assert_eq!(parsed.rpm, 4000);
        assert_eq!(parsed.phase_current, 150);
    }
}

