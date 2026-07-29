// ponytail: Full OEM Kelly KLS parameter interface with sub-tabs, safety modal, read-only grey fields, and JSON export/import.

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use std::collections::{BTreeMap, VecDeque};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

use crate::protocol::kls::KlsTelemetry;
use crate::protocol::oem_params::{
    OemCategory, ParamProfile, ValueType, export_profile_to_json, get_all_param_defs,
    import_profile_from_json,
};
use crate::worker::serial::{WorkerCommand, WorkerEvent, spawn_serial_worker};

#[derive(Debug, PartialEq)]
enum AppTab {
    Dashboard,
    LiveChart,
    Parameters,
    RawLogs,
}

pub struct KlsApp {
    tx_cmd: Sender<WorkerCommand>,
    rx_evt: Receiver<WorkerEvent>,

    available_ports: Vec<String>,
    selected_port: String,
    baud_rate: u32,
    is_connected: bool,
    connected_port_name: String,
    status_msg: String,

    telemetry: KlsTelemetry,

    start_time: Instant,
    voltage_history: VecDeque<[f64; 2]>,
    current_history: VecDeque<[f64; 2]>,
    rpm_history: VecDeque<[f64; 2]>,
    max_history_sec: f64,

    // Parameter State
    oem_subtab: OemCategory,
    param_values: BTreeMap<u8, u16>,
    text_values: BTreeMap<u8, String>,
    pending_write: Option<(u8, u16, &'static str)>, // (addr, val, label)
    show_confirm_modal: bool,

    raw_logs: VecDeque<String>,
    current_tab: AppTab,
}

impl KlsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx_cmd, rx_worker_cmd) = channel();
        let (tx_worker_evt, rx_evt) = channel();

        spawn_serial_worker(rx_worker_cmd, tx_worker_evt);
        let _ = tx_cmd.send(WorkerCommand::ScanPorts);

        let mut param_values = BTreeMap::new();
        for def in get_all_param_defs() {
            param_values.insert(def.addr, def.default_val);
        }

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
            oem_subtab: OemCategory::Vehicle,
            param_values,
            text_values: BTreeMap::new(),
            pending_write: None,
            show_confirm_modal: false,
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
                    self.connected_port_name = port_name;
                    self.status_msg = "Connected".to_string();
                }
                WorkerEvent::Disconnected => {
                    self.is_connected = false;
                    self.status_msg = "Disconnected".to_string();
                }
                WorkerEvent::Telemetry(t) => {
                    let elapsed = self.start_time.elapsed().as_secs_f64();
                    self.voltage_history
                        .push_back([elapsed, t.battery_voltage_v as f64]);
                    self.current_history
                        .push_back([elapsed, t.phase_current_a as f64]);
                    self.rpm_history.push_back([elapsed, t.rpm as f64]);

                    while self
                        .voltage_history
                        .front()
                        .is_some_and(|p| elapsed - p[0] > self.max_history_sec)
                    {
                        self.voltage_history.pop_front();
                        self.current_history.pop_front();
                        self.rpm_history.pop_front();
                    }

                    self.telemetry = t;
                    ctx.request_repaint();
                }
                WorkerEvent::ParamValue { addr, value } => {
                    self.param_values.insert(addr, value as u16);
                    let log = format!("✅ Read/Write Param [0x{:02X}] = {}", addr, value);
                    self.raw_logs.push_back(log);
                    ctx.request_repaint();
                }
                WorkerEvent::RawFrame { is_tx, data } => {
                    let prefix = if is_tx { "TX ->" } else { "RX <-" };
                    let hex_str = data
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.raw_logs.push_back(format!("{} {}", prefix, hex_str));
                    while self.raw_logs.len() > 500 {
                        self.raw_logs.pop_front();
                    }
                }
                WorkerEvent::Error(err) => {
                    self.status_msg = format!("Error: {}", err);
                    self.raw_logs.push_back(format!("❌ Error: {}", err));
                }
            }
        }
    }
}

impl eframe::App for KlsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("⚡ Kelly KLS Companion");
                ui.separator();

                ui.label("Port:");
                egui::ComboBox::from_id_salt("port_select")
                    .selected_text(if self.selected_port.is_empty() {
                        "Select Port"
                    } else {
                        &self.selected_port
                    })
                    .show_ui(ui, |ui| {
                        for p in &self.available_ports {
                            ui.selectable_value(&mut self.selected_port, p.clone(), p);
                        }
                    });

                if ui.button("🔄 Scan").clicked() {
                    let _ = self.tx_cmd.send(WorkerCommand::ScanPorts);
                }

                if self.is_connected {
                    if ui.button("🔴 Disconnect").clicked() {
                        let _ = self.tx_cmd.send(WorkerCommand::Disconnect);
                    }
                } else if ui.button("🟢 Connect").clicked() && !self.selected_port.is_empty() {
                    let _ = self.tx_cmd.send(WorkerCommand::Connect {
                        port_name: self.selected_port.clone(),
                        baud_rate: self.baud_rate,
                    });
                }

                ui.separator();
                ui.label(format!("Status: {}", self.status_msg));
            });
        });

        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, AppTab::Dashboard, "📊 Dashboard");
                ui.selectable_value(&mut self.current_tab, AppTab::LiveChart, "📈 Live Chart");
                ui.selectable_value(
                    &mut self.current_tab,
                    AppTab::Parameters,
                    "⚙ Parameters (OEM)",
                );
                ui.selectable_value(&mut self.current_tab, AppTab::RawLogs, "📜 Serial Logs");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab {
            AppTab::Dashboard => self.show_dashboard(ui),
            AppTab::LiveChart => self.show_live_chart(ui),
            AppTab::Parameters => self.show_parameters(ui, ctx),
            AppTab::RawLogs => self.show_raw_logs(ui),
        });
    }
}

impl KlsApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Telemetry Overview");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("Voltage");
                ui.heading(format!("{:.1} V", self.telemetry.battery_voltage_v));
            });
            ui.group(|ui| {
                ui.label("Phase Current");
                ui.heading(format!("{:.1} A", self.telemetry.phase_current_a));
            });
            ui.group(|ui| {
                ui.label("Motor RPM");
                ui.heading(format!("{} RPM", self.telemetry.rpm));
            });
            ui.group(|ui| {
                ui.label("Ctrl Temp");
                ui.heading(format!("{} °C", self.telemetry.controller_temp_c));
            });
            ui.group(|ui| {
                ui.label("Motor Temp");
                ui.heading(format!("{} °C", self.telemetry.motor_temp_c));
            });
        });
    }

    fn show_live_chart(&mut self, ui: &mut egui::Ui) {
        ui.heading("Real-time Telemetry Plot");
        let v_points: PlotPoints = self.voltage_history.iter().copied().collect();
        let i_points: PlotPoints = self.current_history.iter().copied().collect();
        let rpm_points: PlotPoints = self.rpm_history.iter().copied().collect();

        Plot::new("telemetry_plot")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(v_points)
                        .name("Voltage (V)")
                        .color(egui::Color32::LIGHT_BLUE),
                );
                plot_ui.line(
                    Line::new(i_points)
                        .name("Current (A)")
                        .color(egui::Color32::GOLD),
                );
                plot_ui.line(
                    Line::new(rpm_points)
                        .name("RPM")
                        .color(egui::Color32::GREEN),
                );
            });
    }

    fn show_parameters(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.heading("Kelly KLS Controller Parameters");
            ui.separator();

            if ui.button("📥 Read All").clicked() {
                let addrs: Vec<u8> = get_all_param_defs().iter().map(|d| d.addr).collect();
                let _ = self.tx_cmd.send(WorkerCommand::ReadAllParams(addrs));
            }

            if ui.button("💾 Export JSON").clicked() {
                let profile = ParamProfile {
                    vehicle_name: "KLS-Profile".to_string(),
                    values: self.param_values.clone(),
                    text_values: self.text_values.clone(),
                };
                if let Ok(json) = export_profile_to_json(&profile)
                    && let Some(path) = rfd::FileDialog::new()
                        .set_file_name("kls_params.json")
                        .save_file()
                {
                    let _ = std::fs::write(path, json);
                }
            }

            if ui.button("📂 Import JSON").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
                && let Ok(content) = std::fs::read_to_string(path)
                && let Ok(profile) = import_profile_from_json(&content)
            {
                self.param_values = profile.values;
                self.text_values = profile.text_values;
            }
        });

        ui.add_space(8.0);

        // Sub-tabs: Vehicle, Motor, Control, Features
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.oem_subtab, OemCategory::Vehicle, "🚗 Vehicle");
            ui.selectable_value(&mut self.oem_subtab, OemCategory::Motor, "⚡ Motor");
            ui.selectable_value(&mut self.oem_subtab, OemCategory::Control, "🎛 Control");
            ui.selectable_value(&mut self.oem_subtab, OemCategory::Features, "☑ Features");
        });

        ui.separator();
        ui.add_space(8.0);

        // Disallow write when motor is spinning
        let motor_spinning = self.telemetry.rpm > 0;
        if motor_spinning {
            ui.colored_label(
                egui::Color32::RED,
                "⚠ Motor active (RPM > 0). Parameter writes locked for safety.",
            );
            ui.add_space(5.0);
        }

        ui.horizontal_top(|ui| {
            // Left Column: OEM Parameter Table
            let left_width = (ui.available_width() * 0.50).max(420.0);
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("param_table_scroll")
                        .show(ui, |ui| {
                            egui::Grid::new("param_grid")
                                .striped(true)
                                .num_columns(5)
                                .spacing([12.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("Address").strong());
                                    ui.label(egui::RichText::new("Parameter Name").strong());
                                    ui.label(egui::RichText::new("Value").strong());
                                    ui.label(egui::RichText::new("Unit").strong());
                                    ui.label(egui::RichText::new("Action").strong());

                                    ui.end_row();

                                    for def in get_all_param_defs()
                                        .iter()
                                        .filter(|d| d.category == self.oem_subtab)
                                    {
                                        ui.label(format!("0x{:02X}", def.addr))
                                            .on_hover_text(def.description);
                                        ui.label(def.label).on_hover_text(def.description);

                                        let current_val = *self
                                            .param_values
                                            .get(&def.addr)
                                            .unwrap_or(&def.default_val);

                                        if def.is_read_only {
                                            ui.add_enabled_ui(false, |ui| {
                                                if def.val_type == ValueType::Bool {
                                                    let mut chk = current_val != 0;
                                                    ui.checkbox(&mut chk, "")
                                                        .on_hover_text(def.description);
                                                } else {
                                                    ui.label(format!("{}", current_val))
                                                        .on_hover_text(def.description);
                                                }
                                            });
                                            ui.label(def.unit);
                                            ui.label("🔒 Read-Only").on_hover_text(def.description);
                                        } else {
                                            let mut val = current_val;

                                            if def.val_type == ValueType::Bool {
                                                let mut chk = val != 0;
                                                let label =
                                                    if chk { "Enabled" } else { "Disabled" };
                                                if ui
                                                    .checkbox(&mut chk, label)
                                                    .on_hover_text(def.description)
                                                    .changed()
                                                {
                                                    val = if chk { 1 } else { 0 };
                                                    self.param_values.insert(def.addr, val);
                                                }
                                            } else {
                                                let drag = egui::DragValue::new(&mut val)
                                                    .range(def.min_val..=def.max_val);

                                                if ui
                                                    .add(drag)
                                                    .on_hover_text(def.description)
                                                    .changed()
                                                {
                                                    self.param_values.insert(def.addr, val);
                                                }
                                            }

                                            ui.label(def.unit);

                                            let can_write = self.is_connected && !motor_spinning;
                                            ui.add_enabled_ui(can_write, |ui| {
                                                if ui
                                                    .button("📤 Write")
                                                    .on_hover_text(def.description)
                                                    .clicked()
                                                {
                                                    if def.is_critical {
                                                        self.pending_write =
                                                            Some((def.addr, val, def.label));
                                                        self.show_confirm_modal = true;
                                                    } else {
                                                        let _ = self.tx_cmd.send(
                                                            WorkerCommand::WriteParam {
                                                                addr: def.addr,
                                                                value: val as u8,
                                                            },
                                                        );
                                                    }
                                                }
                                            });
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                },
            );

            ui.separator();

            // Right Column: Interactive Diagram Visualizer Panel
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("visualizer_scroll")
                        .show(ui, |ui| match self.oem_subtab {
                            OemCategory::Vehicle => {
                                crate::visualizer::draw_map_curves(ui, &self.param_values);
                            }
                            OemCategory::Control => {
                                crate::visualizer::draw_pi_step_response(ui, &self.param_values);
                                ui.add_space(8.0);
                                crate::visualizer::draw_rpm_transition(ui, &self.param_values);
                                ui.add_space(8.0);
                                crate::visualizer::draw_ramp_dynamics(ui, &self.param_values);
                            }
                            OemCategory::Motor => {
                                crate::visualizer::draw_thermal_foldback(ui, &self.param_values);
                                ui.add_space(8.0);
                                crate::visualizer::draw_hall_diagram(ui, &self.param_values);
                            }
                            OemCategory::Features => {
                                crate::visualizer::draw_map_curves(ui, &self.param_values);
                            }
                        });
                },
            );
        });

        // Safety Modal Dialog
        if self.show_confirm_modal
            && let Some((addr, val, label)) = self.pending_write
        {
            egui::Window::new("⚠ Confirm Critical Parameter Write")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("You are modifying critical parameter: {}", label));
                    ui.label(format!("Address: 0x{:02X} -> New Value: {}", addr, val));
                    ui.colored_label(
                        egui::Color32::RED,
                        "Warning: Incorrect values may cause motor runaway or hardware failure!",
                    );

                    ui.horizontal(|ui| {
                        if ui.button("✔ Confirm Write").clicked() {
                            let _ = self.tx_cmd.send(WorkerCommand::WriteParam {
                                addr,
                                value: val as u8,
                            });
                            self.show_confirm_modal = false;
                            self.pending_write = None;
                        }
                        if ui.button("❌ Cancel").clicked() {
                            self.show_confirm_modal = false;
                            self.pending_write = None;
                        }
                    });
                });
        }
    }

    fn show_raw_logs(&mut self, ui: &mut egui::Ui) {
        ui.heading("Serial Communication Logs");
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for log in &self.raw_logs {
                    ui.monospace(log);
                }
            });
    }
}
