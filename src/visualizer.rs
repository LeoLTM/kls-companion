// ponytail: Visualizers and diagram generators for Kelly KLS controller parameters.

use eframe::egui;
use egui_plot::{Corner, Legend, Line, LineStyle, Plot, PlotPoints, Points, Text};
use std::collections::BTreeMap;

/// Compute MAP curve point y in [0, 100] for given pedal x in [0, 100]
pub fn calc_map_curve_point(x: f64, dead_low: u16, dead_high: u16, map_pct: u16) -> f64 {
    let dl = (dead_low as f64).clamp(0.0, 59.0);
    let dh = (dead_high as f64).clamp(dl + 1.0, 100.0);
    let m = (map_pct as f64).clamp(0.0, 100.0);

    if x <= dl {
        0.0
    } else if x >= dh {
        100.0
    } else {
        let t = (x - dl) / (dh - dl);
        let a = 200.0 - 4.0 * m;
        let b = 4.0 * m - 100.0;
        let y = a * t * t + b * t;
        y.clamp(0.0, 100.0)
    }
}

/// Simulate 2nd order PI current control loop step response over 20 ms
pub fn simulate_pi_step_response(
    kp_val: u16,
    ki_val: u16,
) -> (Vec<[f64; 2]>, f64, f64, f64, &'static str) {
    let kp = (kp_val as f64 / 1500.0).clamp(0.1, 20.0);
    let ki = (ki_val as f64 / 30.0).clamp(0.01, 50.0);

    let dt = 0.05; // 0.05 ms steps
    let steps = 400; // total 20 ms

    let mut t = 0.0;
    let mut y = 0.0; // output current
    let mut dy = 0.0; // derivative of current
    let mut integral = 0.0;

    let mut points = Vec::with_capacity(steps);
    let mut max_y = 0.0;
    let mut rise_time = 20.0;
    let mut rise_found = false;

    let w0 = 0.8; // natural frequency baseline scale

    for _ in 0..steps {
        let error = 1.0 - y;
        integral += error * dt;
        let u = kp * error + ki * integral * 0.1;

        let d2y = w0 * (u - dy - y);
        dy += d2y * dt;
        y += dy * dt;

        points.push([t, y]);

        if y > max_y {
            max_y = y;
        }

        if !rise_found && y >= 0.9 {
            rise_time = t;
            rise_found = true;
        }

        t += dt;
    }

    let overshoot = ((max_y - 1.0).max(0.0)) * 100.0;

    let mut settling_time = 20.0;
    for p in points.iter().rev() {
        if (p[1] - 1.0).abs() > 0.05 {
            settling_time = p[0];
            break;
        }
    }

    let status = if overshoot > 30.0 {
        "⚠️ High Overshoot (Vibration Risk)"
    } else if overshoot > 15.0 {
        "⚡ Aggressive Response"
    } else if rise_time > 8.0 {
        "🐢 Sluggish Response"
    } else {
        "✅ Stable & Responsive"
    };

    (points, overshoot, rise_time, settling_time, status)
}

/// Render TPS (Throttle) & Brake MAP curves diagram
pub fn draw_map_curves(ui: &mut egui::Ui, param_values: &BTreeMap<u8, u16>) {
    ui.group(|ui| {
        ui.heading("📈 Throttle & Brake Response Curves");
        ui.label(egui::RichText::new("Visualizing pedal input (%) vs controller output (%) with deadzones and 50% midpoint MAP tuning.").small().weak());
        ui.add_space(4.0);

        let tps_dl = *param_values.get(&0x60).unwrap_or(&20);
        let tps_dh = *param_values.get(&0x61).unwrap_or(&80);
        let fwd_map = *param_values.get(&0x62).unwrap_or(&30);
        let rev_map = *param_values.get(&0x63).unwrap_or(&20);

        let brk_dl = *param_values.get(&0x65).unwrap_or(&20);
        let brk_dh = *param_values.get(&0x66).unwrap_or(&80);
        let brk_pct = *param_values.get(&0x9D).unwrap_or(&0);

        let fwd_points: PlotPoints = (0..=100)
            .map(|x| {
                let fx = x as f64;
                [fx, calc_map_curve_point(fx, tps_dl, tps_dh, fwd_map)]
            })
            .collect();

        let rev_points: PlotPoints = (0..=100)
            .map(|x| {
                let fx = x as f64;
                [fx, calc_map_curve_point(fx, tps_dl, tps_dh, rev_map)]
            })
            .collect();

        let brk_points: PlotPoints = (0..=100)
            .map(|x| {
                let fx = x as f64;
                let scale = brk_pct as f64 / 100.0;
                let raw_y = calc_map_curve_point(fx, brk_dl, brk_dh, 50);
                [fx, raw_y * if scale > 0.0 { scale } else { 1.0 }]
            })
            .collect();

        let mid_x = (tps_dl as f64 + tps_dh as f64) / 2.0;
        let mid_fwd_y = calc_map_curve_point(mid_x, tps_dl, tps_dh, fwd_map);

        Plot::new("map_curves_plot")
            .height(240.0)
            .legend(Legend::default().position(Corner::LeftTop))
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(fwd_points)
                        .name("TPS Forward MAP")
                        .color(egui::Color32::GREEN)
                        .width(2.5_f32),
                );
                plot_ui.line(
                    Line::new(rev_points)
                        .name("TPS Reverse MAP")
                        .color(egui::Color32::KHAKI)
                        .width(2.0_f32),
                );
                plot_ui.line(
                    Line::new(brk_points)
                        .name("Brake Regen MAP")
                        .color(egui::Color32::RED)
                        .width(2.0_f32),
                );

                plot_ui.points(
                    Points::new(vec![[mid_x, mid_fwd_y]])
                        .name("50% MAP Point")
                        .color(egui::Color32::YELLOW)
                        .radius(5.0_f32),
                );
            });

        ui.horizontal(|ui| {
            ui.label(format!("FWD MAP @ 50%: {}%", fwd_map));
            ui.separator();
            ui.label(format!("Deadzone: {}% - {}%", tps_dl, tps_dh));
        });
    });
}

/// Render PI controller step response diagram
pub fn draw_pi_step_response(ui: &mut egui::Ui, param_values: &BTreeMap<u8, u16>) {
    ui.group(|ui| {
        ui.heading("🎛 Current Loop PI Step Response Simulation");
        ui.label(
            egui::RichText::new(
                "Simulated phase current step response (I_q / I_d loop) to evaluate Kp/Ki tuning.",
            )
            .small()
            .weak(),
        );
        ui.add_space(4.0);

        let iq_kp = *param_values.get(&0x80).unwrap_or(&1500);
        let iq_ki = *param_values.get(&0x82).unwrap_or(&30);
        let id_kp = *param_values.get(&0x86).unwrap_or(&1500);
        let id_ki = *param_values.get(&0x88).unwrap_or(&30);

        let (iq_points, iq_over, iq_tr, _iq_ts, iq_status) =
            simulate_pi_step_response(iq_kp, iq_ki);
        let (id_points, _id_over, _id_tr, _id_ts, _id_status) =
            simulate_pi_step_response(id_kp, id_ki);

        let target_line: PlotPoints = vec![[0.0, 1.0], [20.0, 1.0]].into();

        Plot::new("pi_step_plot")
            .height(220.0)
            .legend(Legend::default().position(Corner::RightBottom))
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(target_line)
                        .name("Target Step Setpoint")
                        .color(egui::Color32::GRAY)
                        .style(LineStyle::dashed_dense()),
                );
                plot_ui.line(
                    Line::new(iq_points)
                        .name("IQ (Torque Current)")
                        .color(egui::Color32::LIGHT_BLUE)
                        .width(2.5_f32),
                );
                plot_ui.line(
                    Line::new(id_points)
                        .name("ID (Flux Current)")
                        .color(egui::Color32::GOLD)
                        .width(2.0_f32),
                );
            });

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(iq_status).strong());
            ui.separator();
            ui.label(format!(
                "Overshoot: {:.1}% | Rise Time: {:.1} ms",
                iq_over, iq_tr
            ));
        });
        ui.label(
            egui::RichText::new(format!(
                "IQ Kp: {}, Ki: {} | ID Kp: {}, Ki: {}",
                iq_kp, iq_ki, id_kp, id_ki
            ))
            .small()
            .weak(),
        );
    });
}

/// Render Low Speed vs High Speed gain transition visualizer
pub fn draw_rpm_transition(ui: &mut egui::Ui, param_values: &BTreeMap<u8, u16>) {
    ui.group(|ui| {
        ui.heading("🔄 Low vs High Speed Gain Transition (<400 RPM vs >400 RPM)");
        ui.label(
            egui::RichText::new("Kelly controllers switch PI gain sets at 400 RPM threshold.")
                .small()
                .weak(),
        );
        ui.add_space(4.0);

        let iq_kp_low = *param_values.get(&0x80).unwrap_or(&1500) as f64;
        let iq_kp_high = *param_values.get(&0x96).unwrap_or(&1500) as f64;
        let id_kp_low = *param_values.get(&0x86).unwrap_or(&1500) as f64;
        let id_kp_high = *param_values.get(&0x9A).unwrap_or(&1500) as f64;

        let max_speed = *param_values.get(&0x6B).unwrap_or(&4000) as f64;

        let mut kp_q_pts = Vec::new();
        let mut kp_d_pts = Vec::new();

        for rpm in (0..=(max_speed as u32)).step_by(50) {
            let r = rpm as f64;
            let q_val = if r < 400.0 { iq_kp_low } else { iq_kp_high };
            let d_val = if r < 400.0 { id_kp_low } else { id_kp_high };
            kp_q_pts.push([r, q_val]);
            kp_d_pts.push([r, d_val]);
        }

        let max_gain = iq_kp_low
            .max(iq_kp_high)
            .max(id_kp_low)
            .max(id_kp_high)
            .max(100.0);
        let line_400: PlotPoints = vec![[400.0, 0.0], [400.0, max_gain * 1.1]].into();

        Plot::new("rpm_gain_plot")
            .height(180.0)
            .legend(Legend::default().position(Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(line_400)
                        .name("400 RPM Transition Threshold")
                        .color(egui::Color32::RED)
                        .style(LineStyle::dashed_dense()),
                );
                plot_ui.line(
                    Line::new(kp_q_pts)
                        .name("Q-Axis Kp Gain")
                        .color(egui::Color32::LIGHT_BLUE)
                        .width(2.0_f32),
                );
                plot_ui.line(
                    Line::new(kp_d_pts)
                        .name("D-Axis Kp Gain")
                        .color(egui::Color32::GOLD)
                        .width(2.0_f32),
                );
            });
    });
}

/// Render Acceleration and Braking Torque Ramp Dynamics diagram
pub fn draw_ramp_dynamics(ui: &mut egui::Ui, param_values: &BTreeMap<u8, u16>) {
    ui.group(|ui| {
        ui.heading("⏱ Acceleration & Braking Torque Ramp Dynamics");
        ui.label(
            egui::RichText::new("Torque ramp-up and release decay times over time (seconds).")
                .small()
                .weak(),
        );
        ui.add_space(4.0);

        let t_acc = (*param_values.get(&0xA2).unwrap_or(&5) as f64) * 0.1;
        let t_acc_rls = (*param_values.get(&0xA3).unwrap_or(&1) as f64) * 0.1;
        let t_brk = (*param_values.get(&0xA4).unwrap_or(&5) as f64) * 0.1;
        let t_brk_rls = (*param_values.get(&0xA5).unwrap_or(&1) as f64) * 0.1;

        let mut drive_pts = Vec::new();
        for step in 0..200 {
            let t = step as f64 * 0.02;
            let y = if t < 0.2 {
                0.0
            } else if t < 0.2 + t_acc {
                (t - 0.2) / t_acc * 100.0
            } else if t < 2.2 {
                100.0
            } else if t < 2.2 + t_acc_rls {
                (1.0 - (t - 2.2) / t_acc_rls) * 100.0
            } else {
                0.0
            };
            drive_pts.push([t, y.max(0.0)]);
        }

        let mut regen_pts = Vec::new();
        for step in 0..200 {
            let t = step as f64 * 0.02;
            let y = if t < 0.5 {
                0.0
            } else if t < 0.5 + t_brk {
                (t - 0.5) / t_brk * 50.0
            } else if t < 2.0 {
                50.0
            } else if t < 2.0 + t_brk_rls {
                (1.0 - (t - 2.0) / t_brk_rls) * 50.0
            } else {
                0.0
            };
            regen_pts.push([t, y.max(0.0)]);
        }

        Plot::new("ramp_dynamics_plot")
            .height(180.0)
            .legend(Legend::default().position(Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(drive_pts)
                        .name("Drive Acceleration Ramp")
                        .color(egui::Color32::GREEN)
                        .width(2.0_f32),
                );
                plot_ui.line(
                    Line::new(regen_pts)
                        .name("Brake Regen Ramp")
                        .color(egui::Color32::LIGHT_RED)
                        .width(2.0_f32),
                );
            });

        ui.horizontal(|ui| {
            ui.label(format!("Accel Ramp: {:.1}s", t_acc));
            ui.separator();
            ui.label(format!("Accel Release: {:.1}s", t_acc_rls));
            ui.separator();
            ui.label(format!("Brake Ramp: {:.1}s", t_brk));
        });
    });
}

/// Render Motor Thermal Foldback & Over-Temp Protection visualizer
pub fn draw_thermal_foldback(ui: &mut egui::Ui, param_values: &BTreeMap<u8, u16>) {
    ui.group(|ui| {
        ui.heading("🌡 Motor Thermal Current Foldback Curve");
        ui.label(
            egui::RichText::new("Current limiting profile based on motor temperature thresholds.")
                .small()
                .weak(),
        );
        ui.add_space(4.0);

        let t_striae = *param_values.get(&0x47).unwrap_or(&100) as f64;
        let t_cutoff = *param_values.get(&0x46).unwrap_or(&130) as f64;
        let t_resume = *param_values.get(&0x49).unwrap_or(&110) as f64;
        let foldback_pct = *param_values.get(&0x48).unwrap_or(&0) as f64;

        let t_cutoff_effective = t_cutoff.max(t_striae + 1.0);

        let mut foldback_pts = Vec::new();
        for temp in 0..=170 {
            let t = temp as f64;
            let current_limit = if t <= t_striae {
                100.0
            } else if t >= t_cutoff_effective {
                0.0
            } else {
                let ratio = (t - t_striae) / (t_cutoff_effective - t_striae);
                let target_min = if foldback_pct > 0.0 {
                    100.0 - foldback_pct
                } else {
                    0.0
                };
                100.0 - ratio * (100.0 - target_min)
            };
            foldback_pts.push([t, current_limit.max(0.0)]);
        }

        let resume_line: PlotPoints = vec![[t_resume, 0.0], [t_resume, 100.0]].into();

        Plot::new("thermal_foldback_plot")
            .height(180.0)
            .legend(Legend::default().position(Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(foldback_pts)
                        .name("Allowed Current %")
                        .color(egui::Color32::LIGHT_BLUE)
                        .width(2.5_f32),
                );
                plot_ui.line(
                    Line::new(resume_line)
                        .name("Resume Threshold")
                        .color(egui::Color32::GREEN)
                        .style(LineStyle::dashed_dense()),
                );
            });

        ui.horizontal(|ui| {
            ui.label(format!("Foldback Start: {}°C", t_striae as u16));
            ui.separator();
            ui.label(format!("Shutdown: {}°C", t_cutoff as u16));
            ui.separator();
            ui.label(format!("Resume: {}°C", t_resume as u16));
        });
    });
}

/// Render 360° Circular Hall Sensor commutation angle diagram
pub fn draw_hall_diagram(ui: &mut egui::Ui, param_values: &BTreeMap<u8, u16>) {
    ui.group(|ui| {
        ui.heading("🔄 Hall Sensor 6-Step Commutation Diagram");
        ui.label(egui::RichText::new("Electrical angle transitions (0°, 60°, 120°, 180°, 240°, 300°) and auto-identified Hall state codes.").small().weak());
        ui.add_space(4.0);

        let h0 = *param_values.get(&0x06).unwrap_or(&2);
        let h60 = *param_values.get(&0x07).unwrap_or(&6);
        let h120 = *param_values.get(&0x0A).unwrap_or(&4);
        let h180 = *param_values.get(&0x0B).unwrap_or(&5);
        let h240 = *param_values.get(&0x0E).unwrap_or(&1);
        let h300 = *param_values.get(&0x0F).unwrap_or(&3);

        let angles: [(f64, &str, u16); 6] = [
            (0.0, "0°", h0),
            (60.0, "60°", h60),
            (120.0, "120°", h120),
            (180.0, "180°", h180),
            (240.0, "240°", h240),
            (300.0, "300°", h300),
        ];

        let mut circle_pts = Vec::new();
        for i in 0..=360 {
            let rad = (i as f64).to_radians();
            circle_pts.push([rad.cos(), rad.sin()]);
        }

        let mut node_pts = Vec::new();
        for (deg, _, _) in &angles {
            let rad = deg.to_radians();
            node_pts.push([rad.cos(), rad.sin()]);
        }

        Plot::new("hall_diagram_plot")
            .height(200.0)
            .data_aspect(1.0)
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(circle_pts)
                        .color(egui::Color32::DARK_GRAY)
                        .width(1.5_f32),
                );
                plot_ui.points(
                    Points::new(node_pts)
                        .color(egui::Color32::GREEN)
                        .radius(6.0_f32),
                );

                for (deg, label, code) in &angles {
                    let rad = deg.to_radians();
                    let tx = rad.cos() * 1.25;
                    let ty = rad.sin() * 1.25;
                    plot_ui.text(
                        Text::new([tx, ty].into(), format!("{}: Code {}", label, code))
                            .color(egui::Color32::LIGHT_YELLOW),
                    );
                }
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_map_curve_point() {
        // Linear case MAP = 50, Deadzones 0-100
        let y_mid = calc_map_curve_point(50.0, 0, 100, 50);
        assert!((y_mid - 50.0).abs() < 1e-4);

        // Low deadzone 20, high deadzone 80, MAP = 30
        assert_eq!(calc_map_curve_point(10.0, 20, 80, 30), 0.0);
        assert_eq!(calc_map_curve_point(90.0, 20, 80, 30), 100.0);

        // At midpoint 50% pedal (x = 50.0)
        let y_50 = calc_map_curve_point(50.0, 20, 80, 30);
        assert!((y_50 - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_simulate_pi_step_response() {
        let (points, overshoot, rise_time, _settling, _status) =
            simulate_pi_step_response(1500, 30);
        assert_eq!(points.len(), 400);
        assert!(rise_time > 0.0);
        assert!(overshoot >= 0.0);
    }
}
