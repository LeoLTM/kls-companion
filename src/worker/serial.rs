// ponytail: Clean background serial worker handling telemetry polling and parameter read/write commands.

use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::protocol::kls::{
    parse_packet_a, parse_packet_b, KlsCommand, KlsTelemetry, PACKET_LENGTH,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WorkerCommand {
    ScanPorts,
    Connect { port_name: String, baud_rate: u32 },
    Disconnect,
    WriteParam { addr: u8, value: u8 },
    ReadParam { addr: u8 },
    ReadAllParams(Vec<u8>),
    SetPollInterval(Duration),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    PortsDiscovered(Vec<String>),
    Connected(String),
    Disconnected,
    Telemetry(KlsTelemetry),
    ParamValue { addr: u8, value: u8 },
    RawFrame { is_tx: bool, data: Vec<u8> },
    Error(String),
}

fn exchange_packet(
    port: &mut Box<dyn serialport::SerialPort>,
    cmd: u8,
) -> Result<Vec<u8>, String> {
    // Flush input buffer
    let mut flush_buf = [0u8; 1];
    let mut discarded = 0;
    while port.bytes_to_read().unwrap_or(0) > 0 && discarded < 64 {
        let _ = port.read(&mut flush_buf);
        discarded += 1;
    }

    let command_frame = [cmd, 0x00, cmd];
    port.write_all(&command_frame)
        .map_err(|e| format!("TX error (0x{:02X}): {}", cmd, e))?;
    port.flush()
        .map_err(|e| format!("Flush error (0x{:02X}): {}", cmd, e))?;

    let start_time = Instant::now();
    let timeout = Duration::from_millis(150);
    let mut out = vec![0u8; PACKET_LENGTH];
    let mut bytes_read = 0;
    let mut synced = false;
    let mut byte_buf = [0u8; 1];

    while start_time.elapsed() < timeout {
        match port.read(&mut byte_buf) {
            Ok(1) => {
                let b = byte_buf[0];
                if !synced {
                    if b == cmd {
                        out[0] = b;
                        synced = true;
                        bytes_read = 1;
                    }
                } else {
                    out[bytes_read] = b;
                    bytes_read += 1;
                    if bytes_read == PACKET_LENGTH {
                        return Ok(out);
                    }
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => return Err(format!("RX error (0x{:02X}): {}", cmd, e)),
        }
    }

    Err(format!(
        "Timeout reading packet 0x{:02X} (got {}/{} bytes)",
        cmd, bytes_read, PACKET_LENGTH
    ))
}

fn execute_read_param(
    port: &mut Box<dyn serialport::SerialPort>,
    addr: u8,
) -> Result<u8, String> {
    let frame = KlsCommand::ReadParam { addr }.build_frame();
    port.write_all(&frame).map_err(|e| e.to_string())?;
    port.flush().map_err(|e| e.to_string())?;

    let mut buf = [0u8; 5];
    let start = Instant::now();
    let mut read_bytes = 0;

    while start.elapsed() < Duration::from_millis(150) && read_bytes < 5 {
        let mut b = [0u8; 1];
        if let Ok(1) = port.read(&mut b) {
            buf[read_bytes] = b[0];
            read_bytes += 1;
        }
    }

    if read_bytes >= 4 && buf[0] == 0x1B {
        let val = buf[3];
        Ok(val)
    } else {
        Err(format!("Read param 0x{:02X} failed", addr))
    }
}

fn execute_write_param(
    port: &mut Box<dyn serialport::SerialPort>,
    addr: u8,
    value: u8,
) -> Result<(), String> {
    let frame = KlsCommand::WriteParam { addr, value }.build_frame();
    port.write_all(&frame).map_err(|e| e.to_string())?;
    port.flush().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn spawn_serial_worker(
    rx_cmd: Receiver<WorkerCommand>,
    tx_evt: Sender<WorkerEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut port: Option<Box<dyn serialport::SerialPort>> = None;
        let mut poll_interval = Duration::from_millis(50);
        let mut last_poll = Instant::now() - poll_interval;
        let mut telemetry = KlsTelemetry::default();

        loop {
            while let Ok(cmd) = rx_cmd.try_recv() {
                match cmd {
                    WorkerCommand::ScanPorts => {
                        let ports = match serialport::available_ports() {
                            Ok(p) => p.into_iter().map(|p| p.port_name).collect(),
                            Err(_) => Vec::new(),
                        };
                        let _ = tx_evt.send(WorkerEvent::PortsDiscovered(ports));
                    }
                    WorkerCommand::Connect { port_name, baud_rate } => {
                        port = None;
                        match serialport::new(&port_name, baud_rate)
                            .timeout(Duration::from_millis(15))
                            .open()
                        {
                            Ok(p) => {
                                let _ = tx_evt.send(WorkerEvent::Connected(port_name));
                                port = Some(p);
                            }
                            Err(e) => {
                                let _ = tx_evt.send(WorkerEvent::Error(format!(
                                    "Failed to open port: {}",
                                    e
                                )));
                            }
                        }
                    }
                    WorkerCommand::Disconnect => {
                        port = None;
                        let _ = tx_evt.send(WorkerEvent::Disconnected);
                    }
                    WorkerCommand::SetPollInterval(interval) => {
                        poll_interval = interval;
                    }
                    WorkerCommand::WriteParam { addr, value } => {
                        if let Some(ref mut p) = port {
                            let frame = KlsCommand::WriteParam { addr, value }.build_frame();
                            let _ = tx_evt.send(WorkerEvent::RawFrame {
                                is_tx: true,
                                data: frame,
                            });
                            match execute_write_param(p, addr, value) {
                                Ok(_) => {
                                    // Read-back verification
                                    if let Ok(val) = execute_read_param(p, addr) {
                                        let _ = tx_evt.send(WorkerEvent::ParamValue { addr, value: val });
                                    }
                                }
                                Err(e) => {
                                    let _ = tx_evt.send(WorkerEvent::Error(e));
                                }
                            }
                        }
                    }
                    WorkerCommand::ReadParam { addr } => {
                        if let Some(ref mut p) = port {
                            let frame = KlsCommand::ReadParam { addr }.build_frame();
                            let _ = tx_evt.send(WorkerEvent::RawFrame {
                                is_tx: true,
                                data: frame,
                            });
                            match execute_read_param(p, addr) {
                                Ok(val) => {
                                    let _ = tx_evt.send(WorkerEvent::ParamValue { addr, value: val });
                                }
                                Err(e) => {
                                    let _ = tx_evt.send(WorkerEvent::Error(e));
                                }
                            }
                        }
                    }
                    WorkerCommand::ReadAllParams(addrs) => {
                        if let Some(ref mut p) = port {
                            for addr in addrs {
                                if let Ok(val) = execute_read_param(p, addr) {
                                    let _ = tx_evt.send(WorkerEvent::ParamValue { addr, value: val });
                                }
                                thread::sleep(Duration::from_millis(10));
                            }
                        }
                    }
                }
            }

            if let Some(ref mut p) = port {
                if last_poll.elapsed() >= poll_interval {
                    last_poll = Instant::now();
                    let mut updated = false;

                    let tx_a = KlsCommand::QueryPacketA.build_frame();
                    let _ = tx_evt.send(WorkerEvent::RawFrame {
                        is_tx: true,
                        data: tx_a,
                    });

                    match exchange_packet(p, 0x3A) {
                        Ok(bytes) => {
                            let _ = tx_evt.send(WorkerEvent::RawFrame {
                                is_tx: false,
                                data: bytes.clone(),
                            });
                            match parse_packet_a(&bytes) {
                                Ok(pkt_a) => {
                                    telemetry.update_from_packet_a(&pkt_a);
                                    updated = true;
                                }
                                Err(err) => {
                                    let _ = tx_evt.send(WorkerEvent::Error(format!(
                                        "Packet A parse error: {}",
                                        err
                                    )));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = tx_evt.send(WorkerEvent::Error(err));
                        }
                    }

                    let tx_b = KlsCommand::QueryPacketB.build_frame();
                    let _ = tx_evt.send(WorkerEvent::RawFrame {
                        is_tx: true,
                        data: tx_b,
                    });

                    match exchange_packet(p, 0x3B) {
                        Ok(bytes) => {
                            let _ = tx_evt.send(WorkerEvent::RawFrame {
                                is_tx: false,
                                data: bytes.clone(),
                            });
                            match parse_packet_b(&bytes) {
                                Ok(pkt_b) => {
                                    telemetry.update_from_packet_b(&pkt_b);
                                    updated = true;
                                }
                                Err(err) => {
                                    let _ = tx_evt.send(WorkerEvent::Error(format!(
                                        "Packet B parse error: {}",
                                        err
                                    )));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = tx_evt.send(WorkerEvent::Error(err));
                        }
                    }

                    if updated {
                        let _ = tx_evt.send(WorkerEvent::Telemetry(telemetry.clone()));
                    }
                }
            }

            thread::sleep(Duration::from_millis(5));
        }
    })
}
