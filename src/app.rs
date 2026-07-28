// ponytail: Clean single-file UI state management, no unnecessary UI abstractions.

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;

use crate::protocol::kls::KlsTelemetry;
use crate::worker::serial::{spawn_serial_worker, WorkerCommand, WorkerEvent};

#[derive(Debug, PartialEq)]
enum AppTab {
    Dashboard,
    LiveChart,
    Parameters,
    RawLogs,
}

pub struct KlsApp {
    // Worker channels
    tx_cmd: Sender<WorkerCommand>,
    rx_evt: Receiver<WorkerEvent>,

    // Connection state
    available_ports: Vec<String>,
    selected_port: String,
    baud_rate: u32,
    is_connected: bool,
    connected_port_name: String,
    status_msg: String,

    // Telemetry state
    telemetry: KlsTelemetry,

    // Time-series history for plots (timestamp_sec, value)
    start_time: Instant,
    voltage_history: VecDeque<[f64; 2]>,
    current_history: VecDeque<[f64; 2]>,
    rpm_history: VecDeque<[f64; 2]>,
    max_history_sec: f64,

    // Parameter read/write state
    param_addr: u8,
    param_val: u8,
    param_log: Vec<String>,

    // Diagnostic logs
    raw_logs: VecDeque<String>,

    // Selected UI Tab
    current_tab: AppTab,
}

impl KlsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx_cmd, rx_worker_cmd) = channel();
        let (tx_worker_evt, rx_evt) = channel();

        // Spawn background worker thread
        spawn_serial_worker(rx_worker_cmd, tx_worker_evt);

        // Initial scan for ports
        let _ = tx_cmd.send(WorkerCommand::ScanPorts);

        Self {
            tx_cmd,
            rx_evt,
            available_ports: Vec::new(),
            selected_port: String::new(),
            baud_rate: 19200,
            is_connected: false,
            connected_port_name: String::new(),
            status_msg: "Disconnected".to_string(),
            telemetry: KlsTelemetry::default(),
            start_time: Instant::now(),
            voltage_history: VecDeque::with_capacity(1000),
            current_history: VecDeque::with_capacity(1000),
            rpm_history: VecDeque::with_capacity(1000),
            max_history_sec: 60.0,
            param_addr: 0x01,
            param_val: 0x00,
            param_log: Vec::new(),
            raw_logs: VecDeque::with_capacity(500),
            current_tab: AppTab::Dashboard,
        }
    }

    fn poll_events(&mut self, ctx: &egui::Context) {
        while let Ok(evt) = self.rx_evt.try_recv() {
            match evt {
                WorkerEvent::PortsDiscovered(ports) => {
                    self.available_ports = ports;
                    if self.selected_port.is_empty() && !self.available_ports.is_empty() {
                        self.selected_port = self.available_ports[0].clone();
                    }
                }
                WorkerEvent::Connected(port_name) => {
                    self.is_connected = true;
                    self.connected_port_name = port_name.clone();
                    self.status_msg = format!("Connected to {}", port_name);
                }
                WorkerEvent::Disconnected => {
                    self.is_connected = false;
                    self.connected_port_name.clear();
                    self.status_msg = "Disconnected".to_string();
                }
                WorkerEvent::Telemetry(data) => {
                    let elapsed = self.start_time.elapsed().as_secs_f64();

                    // Append to plot histories
                    self.voltage_history
                        .push_back([elapsed, data.battery_voltage_v as f64]);
                    self.current_history
                        .push_back([elapsed, data.phase_current_a as f64]);
                    self.rpm_history.push_back([elapsed, data.rpm as f64]);

                    // Trim old history beyond max_history_sec
                    let cutoff = elapsed - self.max_history_sec;
                    while self
                        .voltage_history
                        .front()
                        .map_or(false, |p| p[0] < cutoff)
                    {
                        self.voltage_history.pop_front();
                    }
                    while self
                        .current_history
                        .front()
                        .map_or(false, |p| p[0] < cutoff)
                    {
                        self.current_history.pop_front();
                    }
                    while self.rpm_history.front().map_or(false, |p| p[0] < cutoff) {
                        self.rpm_history.pop_front();
                    }

                    self.telemetry = data;
                    ctx.request_repaint();
                }
                WorkerEvent::ParamValue { addr, value } => {
                    self.param_log
                        .push(format!("Addr 0x{:02X} = 0x{:02X} ({})", addr, value, value));
                }
                WorkerEvent::RawFrame { is_tx, data } => {
                    let dir = if is_tx { "TX ->" } else { "RX <-" };
                    let hex = data
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    if self.raw_logs.len() >= 500 {
                        self.raw_logs.pop_front();
                    }
                    self.raw_logs.push_back(format!("{} {}", dir, hex));
                }
                WorkerEvent::Error(err) => {
                    self.status_msg = format!("Error: {}", err);
                    if self.raw_logs.len() >= 500 {
                        self.raw_logs.pop_front();
                    }
                    self.raw_logs.push_back(format!("[ERROR] {}", err));
                }
            }
        }
    }
}

impl eframe::App for KlsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events(ctx);

        // Top Header Control Bar
        egui::TopBottomPanel::top("header_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("⚡ Kelly KLS Companion");
                ui.separator();

                // Serial Port Dropdown
                ui.label("Port:");
                egui::ComboBox::from_id_salt("port_combo")
                    .selected_text(if self.selected_port.is_empty() {
                        "Select Port"
                    } else {
                        &self.selected_port
                    })
                    .show_ui(ui, |ui| {
                        for port in &self.available_ports {
                            ui.selectable_value(&mut self.selected_port, port.clone(), port);
                        }
                    });

                if ui.button("🔄 Scan").clicked() {
                    let _ = self.tx_cmd.send(WorkerCommand::ScanPorts);
                }

                ui.separator();
                ui.label("Baud:");
                egui::ComboBox::from_id_salt("baud_combo")
                    .selected_text(format!("{}", self.baud_rate))
                    .show_ui(ui, |ui| {
                        for baud in [9600, 19200, 38400, 57600, 115200] {
                            ui.selectable_value(&mut self.baud_rate, baud, baud.to_string());
                        }
                    });

                ui.separator();

                // Connect / Disconnect Button
                if self.is_connected {
                    if ui.button("🔴 Disconnect").clicked() {
                        let _ = self.tx_cmd.send(WorkerCommand::Disconnect);
                    }
                } else {
                    if ui
                        .add_enabled(!self.selected_port.is_empty(), egui::Button::new("🔌 Connect"))
                        .clicked()
                    {
                        let _ = self.tx_cmd.send(WorkerCommand::Connect {
                            port_name: self.selected_port.clone(),
                            baud_rate: self.baud_rate,
                        });
                    }
                }

                ui.separator();

                // Connection Status Badge
                if self.is_connected {
                    ui.colored_label(egui::Color32::GREEN, "● Connected");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "○ Disconnected");
                }
            });

            ui.separator();

            // Navigation Tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, AppTab::Dashboard, "📊 Dashboard");
                ui.selectable_value(&mut self.current_tab, AppTab::LiveChart, "📈 Live Charts");
                ui.selectable_value(&mut self.current_tab, AppTab::Parameters, "⚙ Parameters");
                ui.selectable_value(&mut self.current_tab, AppTab::RawLogs, "📜 Raw Diagnostics");
            });
        });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Status: {}", self.status_msg));
            });
        });

        // Central View Area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                AppTab::Dashboard => self.show_dashboard(ui),
                AppTab::LiveChart => self.show_live_chart(ui),
                AppTab::Parameters => self.show_parameters(ui),
                AppTab::RawLogs => self.show_raw_logs(ui),
            }
        });

        // Trigger continuous UI updates when connected for smooth live telemetry
        if self.is_connected {
            ctx.request_repaint();
        }
    }
}

impl KlsApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Live Controller Status");
        ui.add_space(8.0);

        egui::Grid::new("telemetry_grid")
            .num_columns(3)
            .spacing([20.0, 15.0])
            .show(ui, |ui| {
                // Battery Voltage
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Battery Voltage");
                        ui.heading(format!("{:.1} V", self.telemetry.battery_voltage_v));
                    });
                });

                // Phase Current
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Phase Current");
                        ui.heading(format!("{:.1} A", self.telemetry.phase_current_a));
                    });
                });

                // Motor RPM
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Motor RPM");
                        ui.heading(format!("{} RPM", self.telemetry.rpm));
                    });
                });

                ui.end_row();

                // Controller Temp
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Controller Temp");
                        ui.heading(format!("{} °C", self.telemetry.controller_temp_c));
                    });
                });

                // Motor Temp
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Motor Temp");
                        ui.heading(format!("{} °C", self.telemetry.motor_temp_c));
                    });
                });

                // Error Code
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Error Status");
                        if self.telemetry.error_code == 0 {
                            ui.colored_label(egui::Color32::GREEN, "✔ Normal (0x00)");
                        } else {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("⚠ Fault: 0x{:04X}", self.telemetry.error_code),
                            );
                        }
                    });
                });

                ui.end_row();
            });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(10.0);

        // Throttle & Brake Bars
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(format!("Throttle: {}% (Raw: {})", self.telemetry.throttle_pct, self.telemetry.throttle));
                ui.add(egui::ProgressBar::new(self.telemetry.throttle_pct as f32 / 100.0));

                ui.add_space(5.0);

                ui.label(format!("Brake Pedal: {}% (Raw: {})", self.telemetry.brake_pct, self.telemetry.brake_pedal));
                ui.add(egui::ProgressBar::new(self.telemetry.brake_pct as f32 / 100.0));
            });
        });

        ui.add_space(10.0);

        // Switches & Hall Status
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Digital Inputs & Hall Sensors");
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    let pill = |ui: &mut egui::Ui, label: &str, active: bool| {
                        if active {
                            ui.colored_label(egui::Color32::GREEN, format!("🟢 {}", label));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, format!("⚪ {}", label));
                        }
                    };

                    pill(ui, "Fwd Sw", self.telemetry.forward_switch);
                    ui.add_space(10.0);
                    pill(ui, "Rev Sw", self.telemetry.reverse);
                    ui.add_space(10.0);
                    pill(ui, "Brake Sw", self.telemetry.brake_switch);
                    ui.add_space(10.0);
                    pill(ui, "Foot Sw", self.telemetry.foot_switch);
                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(15.0);
                    pill(ui, "Hall A", self.telemetry.hall_a);
                    ui.add_space(10.0);
                    pill(ui, "Hall B", self.telemetry.hall_b);
                    ui.add_space(10.0);
                    pill(ui, "Hall C", self.telemetry.hall_c);
                });
            });
        });
    }

    fn show_live_chart(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Real-time Telemetry Plot");
            ui.separator();
            ui.label("Time Window:");
            ui.add(egui::Slider::new(&mut self.max_history_sec, 10.0..=300.0).suffix(" s"));
        });

        ui.add_space(8.0);

        let v_points: PlotPoints = self.voltage_history.iter().copied().collect();
        let i_points: PlotPoints = self.current_history.iter().copied().collect();
        let rpm_points: PlotPoints = self.rpm_history.iter().copied().collect();

        Plot::new("telemetry_plot")
            .view_aspect(2.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(v_points).name("Voltage (V)").color(egui::Color32::LIGHT_BLUE));
                plot_ui.line(Line::new(i_points).name("Current (A)").color(egui::Color32::GOLD));
                plot_ui.line(Line::new(rpm_points).name("RPM").color(egui::Color32::GREEN));
            });
    }

    fn show_parameters(&mut self, ui: &mut egui::Ui) {
        ui.heading("Parameter Read / Write (KLS Command 0x1B / 0x42)");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Address (Hex): 0x");
            ui.add(egui::DragValue::new(&mut self.param_addr).hexadecimal(2, false, true));

            ui.separator();

            if ui.button("📥 Read Param").clicked() {
                let _ = self.tx_cmd.send(WorkerCommand::ReadParam {
                    addr: self.param_addr,
                });
            }
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Value (Hex): 0x");
            ui.add(egui::DragValue::new(&mut self.param_val).hexadecimal(2, false, true));

            ui.separator();

            if ui.button("📤 Write Param").clicked() {
                let _ = self.tx_cmd.send(WorkerCommand::WriteParam {
                    addr: self.param_addr,
                    value: self.param_val,
                });
            }
        });

        ui.add_space(15.0);
        ui.separator();
        ui.label("Parameter Log:");

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for log in &self.param_log {
                    ui.label(log);
                }
            });
    }

    fn show_raw_logs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Raw Serial Diagnostics");
            ui.separator();
            if ui.button("🗑 Clear Logs").clicked() {
                self.raw_logs.clear();
            }
        });

        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for log in &self.raw_logs {
                    ui.monospace(log);
                }
            });
    }
}
