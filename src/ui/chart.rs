use crate::engine::runner::Runner;
use egui::{pos2, vec2, Align2, Color32, FontId, Pos2, RichText, Sense, Stroke, Ui, Window};
use egui_plot::{Bar, BarChart, Legend, Line, Plot, Points};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartDimension {
    Time1D,       // 1D Time series (Step vs Variable/Output)
    Parametric2D, // 2D Parametric (X Variable vs Y Variable)
    Spatial3D,    // 3D Spatial (X Variable vs Y Variable vs Z Variable)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartSeriesType {
    Outputs,
    Variable(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotType {
    Line,
    Points,
    Bar,
}

pub struct ChartWindowState {
    pub is_open: bool,
    pub dimension: ChartDimension,

    // 1D Mode fields
    pub series_1d: ChartSeriesType,
    pub plot_type: PlotType,

    // 2D Mode fields
    pub var_x_2d: String,
    pub var_y_2d: String,

    // 3D Mode fields
    pub var_x_3d: String,
    pub var_y_3d: String,
    pub var_z_3d: String,

    // 3D Orbit Camera State
    pub camera_pitch: f32,
    pub camera_yaw: f32,
    pub camera_zoom: f32,
}

impl Default for ChartWindowState {
    fn default() -> Self {
        Self {
            is_open: false,
            dimension: ChartDimension::Time1D,
            series_1d: ChartSeriesType::Outputs,
            plot_type: PlotType::Line,
            var_x_2d: "x".to_string(),
            var_y_2d: "y".to_string(),
            var_x_3d: "x".to_string(),
            var_y_3d: "y".to_string(),
            var_z_3d: "z".to_string(),
            camera_pitch: 0.4,
            camera_yaw: 0.6,
            camera_zoom: 1.0,
        }
    }
}

pub fn render_chart_window(
    ctx: &egui::Context,
    runner: &Runner,
    state: &mut ChartWindowState,
) {
    if !state.is_open {
        return;
    }

    let mut is_open = state.is_open;

    Window::new("📊 Algoritma Grafik ve 3D Analiz Paneli")
        .open(&mut is_open)
        .default_size([720.0, 520.0])
        .min_size([450.0, 350.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Top Control Bar: Mode Tabs (1D, 2D, 3D) & CSV Export
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Mod:").strong().size(12.0));
                    ui.selectable_value(&mut state.dimension, ChartDimension::Time1D, "📈 1D (Zaman)");
                    ui.selectable_value(&mut state.dimension, ChartDimension::Parametric2D, "📊 2D (X - Y)");
                    ui.selectable_value(&mut state.dimension, ChartDimension::Spatial3D, "🧊 3D (X - Y - Z)");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(RichText::new("💾 CSV Dışa Aktar").size(11.0))
                            .on_hover_text("Seçili grafik verilerini CSV dosyası olarak kaydet")
                            .clicked()
                        {
                            export_csv_data(runner, state);
                        }
                    });
                });

                ui.separator();

                // Render active dimension mode UI
                match state.dimension {
                    ChartDimension::Time1D => render_1d_mode(ui, runner, state),
                    ChartDimension::Parametric2D => render_2d_mode(ui, runner, state),
                    ChartDimension::Spatial3D => render_3d_mode(ui, runner, state),
                }
            });
        });

    state.is_open = is_open;
}

fn render_1d_mode(ui: &mut Ui, runner: &Runner, state: &mut ChartWindowState) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Veri Serisi:").strong().size(12.0));

        let current_label = match &state.series_1d {
            ChartSeriesType::Outputs => "📊 Çıktılar (Outputs)".to_string(),
            ChartSeriesType::Variable(name) => format!("📈 Değişken: {}", name),
        };

        egui::ComboBox::from_id_salt("chart_1d_series_select")
            .selected_text(current_label)
            .width(180.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.series_1d,
                    ChartSeriesType::Outputs,
                    "📊 Çıktılar (Outputs)",
                );

                let mut var_names: Vec<String> = runner.variable_history.keys().cloned().collect();
                var_names.sort();

                for name in var_names {
                    ui.selectable_value(
                        &mut state.series_1d,
                        ChartSeriesType::Variable(name.clone()),
                        format!("📈 Değişken: {}", name),
                    );
                }
            });

        ui.separator();

        ui.label(RichText::new("Grafik Tipi:").strong().size(12.0));
        ui.selectable_value(&mut state.plot_type, PlotType::Line, "📈 Çizgi");
        ui.selectable_value(&mut state.plot_type, PlotType::Bar, "📊 Çubuk");
        ui.selectable_value(&mut state.plot_type, PlotType::Points, "📍 Nokta");
    });

    ui.separator();

    let raw_data: Vec<(usize, f64)> = match &state.series_1d {
        ChartSeriesType::Outputs => runner.output_history.clone(),
        ChartSeriesType::Variable(name) => runner
            .variable_history
            .get(name)
            .cloned()
            .unwrap_or_default(),
    };

    if raw_data.is_empty() {
        render_empty_notice(ui);
        return;
    }

    render_summary_stats(ui, &raw_data);
    ui.separator();

    Plot::new("plot_1d")
        .view_aspect(2.2)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            let points: Vec<[f64; 2]> = raw_data
                .iter()
                .enumerate()
                .map(|(i, &(_step, y))| [i as f64, y])
                .collect();

            match state.plot_type {
                PlotType::Line => {
                    plot_ui.line(Line::new(points).color(Color32::from_rgb(45, 212, 191)).width(2.5).name("1D Değer"));
                }
                PlotType::Points => {
                    plot_ui.points(Points::new(points).color(Color32::from_rgb(251, 146, 60)).radius(4.5).name("1D Değer"));
                }
                PlotType::Bar => {
                    let bars: Vec<Bar> = raw_data
                        .iter()
                        .enumerate()
                        .map(|(i, &(_step, y))| Bar::new(i as f64, y).width(0.6))
                        .collect();
                    plot_ui.bar_chart(BarChart::new(bars).color(Color32::from_rgb(99, 102, 241)).name("1D Değer"));
                }
            }
        });
}

fn render_2d_mode(ui: &mut Ui, runner: &Runner, state: &mut ChartWindowState) {
    let var_names: Vec<String> = {
        let mut v: Vec<String> = runner.variable_history.keys().cloned().collect();
        v.sort();
        v
    };

    if var_names.is_empty() {
        render_empty_notice(ui);
        return;
    }

    // Set defaults if current variables don't exist
    if !var_names.contains(&state.var_x_2d) {
        state.var_x_2d = var_names[0].clone();
    }
    if !var_names.contains(&state.var_y_2d) {
        state.var_y_2d = var_names.get(1).cloned().unwrap_or_else(|| var_names[0].clone());
    }

    ui.horizontal(|ui| {
        ui.label(RichText::new("X Eksen Değişkeni:").strong().size(12.0));
        egui::ComboBox::from_id_salt("combo_2d_x")
            .selected_text(&state.var_x_2d)
            .width(120.0)
            .show_ui(ui, |ui| {
                for name in &var_names {
                    ui.selectable_value(&mut state.var_x_2d, name.clone(), name);
                }
            });

        ui.separator();

        ui.label(RichText::new("Y Eksen Değişkeni:").strong().size(12.0));
        egui::ComboBox::from_id_salt("combo_2d_y")
            .selected_text(&state.var_y_2d)
            .width(120.0)
            .show_ui(ui, |ui| {
                for name in &var_names {
                    ui.selectable_value(&mut state.var_y_2d, name.clone(), name);
                }
            });
    });

    ui.separator();

    let x_pts = runner.variable_history.get(&state.var_x_2d);
    let y_pts = runner.variable_history.get(&state.var_y_2d);

    if x_pts.is_none() || y_pts.is_none() {
        render_empty_notice(ui);
        return;
    }

    let xs = x_pts.unwrap();
    let ys = y_pts.unwrap();
    let count = xs.len().min(ys.len());

    let points_2d: Vec<[f64; 2]> = (0..count).map(|i| [xs[i].1, ys[i].1]).collect();

    Plot::new("plot_2d")
        .view_aspect(2.2)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(points_2d.clone())
                    .color(Color32::from_rgb(45, 212, 191))
                    .width(2.5)
                    .name(format!("{} vs {}", state.var_x_2d, state.var_y_2d)),
            );
            plot_ui.points(
                Points::new(points_2d)
                    .color(Color32::from_rgb(251, 146, 60))
                    .radius(4.0)
                    .name("Noktalar"),
            );
        });
}

fn render_3d_mode(ui: &mut Ui, runner: &Runner, state: &mut ChartWindowState) {
    let var_names: Vec<String> = {
        let mut v: Vec<String> = runner.variable_history.keys().cloned().collect();
        v.sort();
        v
    };

    if var_names.is_empty() {
        render_empty_notice(ui);
        return;
    }

    // Set defaults if current variables don't exist
    if !var_names.contains(&state.var_x_3d) {
        state.var_x_3d = var_names[0].clone();
    }
    if !var_names.contains(&state.var_y_3d) {
        state.var_y_3d = var_names.get(1).cloned().unwrap_or_else(|| var_names[0].clone());
    }
    if !var_names.contains(&state.var_z_3d) {
        state.var_z_3d = var_names.get(2).cloned().unwrap_or_else(|| var_names[0].clone());
    }

    ui.horizontal(|ui| {
        ui.label(RichText::new("X (Kırmızı):").strong().size(11.0).color(Color32::from_rgb(248, 113, 113)));
        egui::ComboBox::from_id_salt("combo_3d_x")
            .selected_text(&state.var_x_3d)
            .width(90.0)
            .show_ui(ui, |ui| {
                for name in &var_names {
                    ui.selectable_value(&mut state.var_x_3d, name.clone(), name);
                }
            });

        ui.label(RichText::new("Y (Yeşil):").strong().size(11.0).color(Color32::from_rgb(74, 222, 128)));
        egui::ComboBox::from_id_salt("combo_3d_y")
            .selected_text(&state.var_y_3d)
            .width(90.0)
            .show_ui(ui, |ui| {
                for name in &var_names {
                    ui.selectable_value(&mut state.var_y_3d, name.clone(), name);
                }
            });

        ui.label(RichText::new("Z (Mavi):").strong().size(11.0).color(Color32::from_rgb(96, 165, 250)));
        egui::ComboBox::from_id_salt("combo_3d_z")
            .selected_text(&state.var_z_3d)
            .width(90.0)
            .show_ui(ui, |ui| {
                for name in &var_names {
                    ui.selectable_value(&mut state.var_z_3d, name.clone(), name);
                }
            });

        if ui.button(RichText::new("🎯 Kamera").size(11.0)).on_hover_text("3D kamerayı sıfırla").clicked() {
            state.camera_pitch = 0.4;
            state.camera_yaw = 0.6;
            state.camera_zoom = 1.0;
        }
    });

    ui.separator();

    // Render interactive 3D perspective canvas viewport
    render_3d_viewport(ui, runner, state);
}

fn render_3d_viewport(ui: &mut Ui, runner: &Runner, state: &mut ChartWindowState) {
    let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 350.0), Sense::drag());
    let rect = response.rect;

    // Draw background 3D viewport canvas
    painter.rect_filled(rect, 6.0, Color32::from_rgb(15, 23, 42));
    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)));

    // Orbit Camera controls
    if response.dragged() {
        let delta = response.drag_delta();
        state.camera_yaw += delta.x * 0.01;
        state.camera_pitch = (state.camera_pitch + delta.y * 0.01).clamp(-1.4, 1.4);
    }

    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            let factor = if scroll > 0.0 { 1.1 } else { 0.9 };
            state.camera_zoom = (state.camera_zoom * factor).clamp(0.2, 5.0);
        }
    }

    // Collect 3D data points
    let x_data = runner.variable_history.get(&state.var_x_3d);
    let y_data = runner.variable_history.get(&state.var_y_3d);
    let z_data = runner.variable_history.get(&state.var_z_3d);

    if x_data.is_none() || y_data.is_none() || z_data.is_none() {
        return;
    }

    let xs = x_data.unwrap();
    let ys = y_data.unwrap();
    let zs = z_data.unwrap();

    let count = xs.len().min(ys.len()).min(zs.len());
    if count == 0 {
        return;
    }

    // Compute min/max for normalization
    let mut min_x = f64::INFINITY; let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY; let mut max_y = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY; let mut max_z = f64::NEG_INFINITY;

    for i in 0..count {
        min_x = min_x.min(xs[i].1); max_x = max_x.max(xs[i].1);
        min_y = min_y.min(ys[i].1); max_y = max_y.max(ys[i].1);
        min_z = min_z.min(zs[i].1); max_z = max_z.max(zs[i].1);
    }

    let range_x = if (max_x - min_x).abs() < 1e-6 { 1.0 } else { max_x - min_x };
    let range_y = if (max_y - min_y).abs() < 1e-6 { 1.0 } else { max_y - min_y };
    let range_z = if (max_z - min_z).abs() < 1e-6 { 1.0 } else { max_z - min_z };

    let center = rect.center();
    let scale = (rect.height().min(rect.width()) * 0.35) * state.camera_zoom;

    let project_3d = |x_norm: f32, y_norm: f32, z_norm: f32| -> (Pos2, f32) {
        // Yaw (around Y axis)
        let x1 = x_norm * state.camera_yaw.cos() - z_norm * state.camera_yaw.sin();
        let z1 = x_norm * state.camera_yaw.sin() + z_norm * state.camera_yaw.cos();

        // Pitch (around X axis)
        let y2 = y_norm * state.camera_pitch.cos() - z1 * state.camera_pitch.sin();
        let z2 = y_norm * state.camera_pitch.sin() + z1 * state.camera_pitch.cos();

        (pos2(center.x + x1 * scale, center.y - y2 * scale), z2)
    };

    // Draw 3D Axes (X=Red, Y=Green, Z=Blue)
    let (o2d, _) = project_3d(0.0, 0.0, 0.0);
    let (x2d, _) = project_3d(1.0, 0.0, 0.0);
    let (y2d, _) = project_3d(0.0, 1.0, 0.0);
    let (z2d, _) = project_3d(0.0, 0.0, 1.0);

    painter.line_segment([o2d, x2d], Stroke::new(2.5, Color32::from_rgb(248, 113, 113)));
    painter.line_segment([o2d, y2d], Stroke::new(2.5, Color32::from_rgb(74, 222, 128)));
    painter.line_segment([o2d, z2d], Stroke::new(2.5, Color32::from_rgb(96, 165, 250)));

    painter.text(x2d, Align2::LEFT_TOP, format!("X ({})", state.var_x_3d), FontId::proportional(11.0), Color32::from_rgb(248, 113, 113));
    painter.text(y2d, Align2::CENTER_BOTTOM, format!("Y ({})", state.var_y_3d), FontId::proportional(11.0), Color32::from_rgb(74, 222, 128));
    painter.text(z2d, Align2::RIGHT_TOP, format!("Z ({})", state.var_z_3d), FontId::proportional(11.0), Color32::from_rgb(96, 165, 250));

    // Project 3D data trajectory
    let mut projected_pts = Vec::with_capacity(count);
    for i in 0..count {
        let nx = (((xs[i].1 - min_x) / range_x) * 1.6 - 0.8) as f32;
        let ny = (((ys[i].1 - min_y) / range_y) * 1.6 - 0.8) as f32;
        let nz = (((zs[i].1 - min_z) / range_z) * 1.6 - 0.8) as f32;
        let (screen_pos, depth) = project_3d(nx, ny, nz);
        projected_pts.push((screen_pos, depth));
    }

    // Draw 3D trajectory line
    for i in 0..count.saturating_sub(1) {
        let p1 = projected_pts[i].0;
        let p2 = projected_pts[i + 1].0;
        painter.line_segment([p1, p2], Stroke::new(2.0, Color32::from_rgb(45, 212, 191)));
    }

    // Draw 3D data point spheres
    for (pos, _depth) in &projected_pts {
        painter.circle_filled(*pos, 3.5, Color32::from_rgb(251, 146, 60));
        painter.circle_stroke(*pos, 3.5, Stroke::new(1.0, Color32::WHITE));
    }

    // Controls tip
    painter.text(
        pos2(rect.min.x + 10.0, rect.max.y - 14.0),
        Align2::LEFT_BOTTOM,
        "💡 3D Kamerayı Sürükleyerek Döndürün | Fare Tekerleği ile Yakınlaşın",
        FontId::proportional(11.0),
        Color32::from_rgb(100, 116, 139),
    );
}

fn render_empty_notice(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        ui.label(
            RichText::new("Seçilen mod ve değişkenler için henüz sayısal veri bulunamadı.")
                .color(Color32::from_rgb(148, 163, 184))
                .size(14.0),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("Diyagramı çalıştırın veya döngü içeren bir algoritma yürütün (örn: cos(x), Fibonacci, 3D Yörünge).")
                .color(Color32::from_rgb(100, 116, 139))
                .size(12.0),
        );
    });
}

fn render_summary_stats(ui: &mut Ui, data: &[(usize, f64)]) {
    let count = data.len();
    let values: Vec<f64> = data.iter().map(|(_, y)| *y).collect();
    let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let avg_val = values.iter().sum::<f64>() / count as f64;

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("🔢 Toplam Adım: {}", count)).size(11.0));
        ui.separator();
        ui.label(RichText::new(format!("⬇️ Min: {:.4}", min_val)).size(11.0));
        ui.separator();
        ui.label(RichText::new(format!("⬆️ Max: {:.4}", max_val)).size(11.0));
        ui.separator();
        ui.label(RichText::new(format!("⚖️ Ort: {:.4}", avg_val)).size(11.0));
    });
}

fn export_csv_data(runner: &Runner, state: &ChartWindowState) {
    match state.dimension {
        ChartDimension::Time1D => {
            let data: Vec<(usize, f64)> = match &state.series_1d {
                ChartSeriesType::Outputs => runner.output_history.clone(),
                ChartSeriesType::Variable(name) => runner
                    .variable_history
                    .get(name)
                    .cloned()
                    .unwrap_or_default(),
            };

            if !data.is_empty() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("chart_1d.csv")
                    .add_filter("CSV Dosyası", &["csv"])
                    .save_file()
                {
                    let mut csv_lines = vec!["Step,Value".to_string()];
                    for (step, val) in &data {
                        csv_lines.push(format!("{},{}", step, val));
                    }
                    let _ = std::fs::write(path, csv_lines.join("\n"));
                }
            }
        }
        ChartDimension::Parametric2D => {
            let xs = runner.variable_history.get(&state.var_x_2d);
            let ys = runner.variable_history.get(&state.var_y_2d);
            if let (Some(x_data), Some(y_data)) = (xs, ys) {
                let count = x_data.len().min(y_data.len());
                if count > 0 {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("chart_2d.csv")
                        .add_filter("CSV Dosyası", &["csv"])
                        .save_file()
                    {
                        let mut csv_lines = vec![format!("{},{}", state.var_x_2d, state.var_y_2d)];
                        for i in 0..count {
                            csv_lines.push(format!("{},{}", x_data[i].1, y_data[i].1));
                        }
                        let _ = std::fs::write(path, csv_lines.join("\n"));
                    }
                }
            }
        }
        ChartDimension::Spatial3D => {
            let xs = runner.variable_history.get(&state.var_x_3d);
            let ys = runner.variable_history.get(&state.var_y_3d);
            let zs = runner.variable_history.get(&state.var_z_3d);
            if let (Some(x_data), Some(y_data), Some(z_data)) = (xs, ys, zs) {
                let count = x_data.len().min(y_data.len()).min(z_data.len());
                if count > 0 {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("chart_3d.csv")
                        .add_filter("CSV Dosyası", &["csv"])
                        .save_file()
                    {
                        let mut csv_lines = vec![format!(
                            "{},{},{}",
                            state.var_x_3d, state.var_y_3d, state.var_z_3d
                        )];
                        for i in 0..count {
                            csv_lines.push(format!(
                                "{},{},{}",
                                x_data[i].1, y_data[i].1, z_data[i].1
                            ));
                        }
                        let _ = std::fs::write(path, csv_lines.join("\n"));
                    }
                }
            }
        }
    }
}
