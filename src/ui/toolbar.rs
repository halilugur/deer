use crate::engine::runner::{ExecutionState, Runner};
use crate::model::diagram::Diagram;
use egui::{vec2, Align, Button, Color32, ComboBox, Frame, Image, Layout, Margin, RichText, TextureHandle, Ui};
use std::path::Path;

pub struct ToolbarAction {
    pub new_diagram: bool,
    pub open_file: bool,
    pub save_file: bool,
    pub save_file_as: bool,
    pub center_view: bool,
}

fn render_toolbar_icon_btn(
    ui: &mut Ui,
    icon_key: &'static str,
    svg_bytes: &'static [u8],
    label: &str,
    fill_color: Option<Color32>,
    text_color: Option<Color32>,
    tooltip: &str,
) -> bool {
    let text_col = text_color.unwrap_or_else(|| Color32::from_rgb(226, 232, 240));

    if let Ok(img) = egui_extras::image::load_svg_bytes(svg_bytes) {
        let tex = ui.ctx().load_texture(icon_key, img, egui::TextureOptions::LINEAR);
        let image = Image::new(&tex).fit_to_exact_size(vec2(14.0, 14.0));
        let mut btn = Button::image_and_text(image, RichText::new(label).size(12.0).strong().color(text_col));
        if let Some(fill) = fill_color {
            btn = btn.fill(fill);
        }
        ui.add(btn).on_hover_text(tooltip).clicked()
    } else {
        let mut btn = Button::new(RichText::new(label).size(12.0).strong().color(text_col));
        if let Some(fill) = fill_color {
            btn = btn.fill(fill);
        }
        ui.add(btn).on_hover_text(tooltip).clicked()
    }
}

pub fn render_toolbar(
    ui: &mut Ui,
    _diagram: &Diagram,
    runner: &mut Runner,
    show_vars: &mut bool,
    show_stack: &mut bool,
    show_console: &mut bool,
    show_chart: &mut bool,
    is_dark: &mut bool,
    current_filepath: Option<&Path>,
    logo_texture: Option<&TextureHandle>,
) -> ToolbarAction {
    let mut action = ToolbarAction {
        new_diagram: false,
        open_file: false,
        save_file: false,
        save_file_as: false,
        center_view: false,
    };

    let is_running = runner.state == ExecutionState::Running;
    let is_paused = runner.state == ExecutionState::Paused;
    let is_waiting = matches!(runner.state, ExecutionState::WaitingForInput { .. });

    // Custom frame padding matching DEER monochromatic theme
    Frame::none()
        .inner_margin(Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // 1. BRANDING WITH OFFICIAL LOGO & FILE TITLE
                ui.horizontal(|ui| {
                    if let Some(tex) = logo_texture {
                        ui.add(Image::new(tex).fit_to_exact_size(vec2(22.0, 22.0)));
                    }

                    ui.label(RichText::new("DEER").size(15.0).strong().color(Color32::from_rgb(226, 232, 240)));
                    ui.label(
                        RichText::new("ENGINE")
                            .size(9.0)
                            .strong()
                            .color(Color32::from_rgb(138, 150, 164)),
                    );

                    ui.add_space(4.0);

                    // File badge indicator
                    let file_label = if let Some(path) = current_filepath {
                        format!("Dosya: {}", path.file_name().unwrap_or_default().to_string_lossy())
                    } else {
                        "Yeni Diyagram".to_string()
                    };

                    ui.label(
                        RichText::new(file_label)
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // 2. FILE ACTION BUTTONS (New, Open, Save, Save As)
                if render_toolbar_icon_btn(
                    ui,
                    "tb_new",
                    include_bytes!("../../assets/icons/new_doc.svg"),
                    "Yeni",
                    None,
                    None,
                    "Yeni bir diyagram oluşturur",
                ) {
                    action.new_diagram = true;
                }

                if render_toolbar_icon_btn(
                    ui,
                    "tb_open",
                    include_bytes!("../../assets/icons/open_doc.svg"),
                    "Aç",
                    None,
                    None,
                    "Var olan bir diyagram (.dfpp/.fpp) dosyasını açar",
                ) {
                    action.open_file = true;
                }

                if render_toolbar_icon_btn(
                    ui,
                    "tb_save",
                    include_bytes!("../../assets/icons/save_doc.svg"),
                    "Kaydet",
                    None,
                    None,
                    "Mevcut diyagramı varsayılan dosyaya kaydeder",
                ) {
                    action.save_file = true;
                }

                if render_toolbar_icon_btn(
                    ui,
                    "tb_save_as",
                    include_bytes!("../../assets/icons/save_doc.svg"),
                    "Farklı Kaydet",
                    None,
                    None,
                    "Diyagramı yeni bir dosya adı ile kaydeder",
                ) {
                    action.save_file_as = true;
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // 3. RUNNER CONTROLS (Play, Pause, Stop, Step, Speed)
                ui.horizontal(|ui| {
                    if is_running {
                        if render_toolbar_icon_btn(
                            ui,
                            "tb_pause",
                            include_bytes!("../../assets/icons/pause.svg"),
                            "Duraklat",
                            Some(Color32::from_rgb(250, 204, 21)),
                            Some(Color32::from_rgb(20, 20, 20)),
                            "Çalışmayı geçici olarak duraklatır",
                        ) {
                            runner.pause();
                        }
                    } else if is_paused {
                        if render_toolbar_icon_btn(
                            ui,
                            "tb_resume",
                            include_bytes!("../../assets/icons/play.svg"),
                            "Devam Et",
                            Some(Color32::from_rgb(45, 212, 191)),
                            Some(Color32::from_rgb(15, 23, 42)),
                            "Duraklatılan çalışmaya devam eder",
                        ) {
                            runner.resume();
                        }
                    } else if render_toolbar_icon_btn(
                        ui,
                        "tb_play",
                        include_bytes!("../../assets/icons/play.svg"),
                        "Çalıştır",
                        Some(Color32::from_rgb(45, 212, 191)),
                        Some(Color32::from_rgb(15, 23, 42)),
                        "Diyagramı sıfırdan çalıştırır",
                    ) {
                        runner.start(_diagram);
                    }

                    if (is_running || is_paused || is_waiting)
                        && render_toolbar_icon_btn(
                            ui,
                            "tb_stop",
                            include_bytes!("../../assets/icons/stop.svg"),
                            "Durdur",
                            Some(Color32::from_rgb(248, 113, 113)),
                            Some(Color32::WHITE),
                            "Çalışmayı tamamen sonlandırır",
                        )
                    {
                        runner.stop();
                    }

                    if render_toolbar_icon_btn(
                        ui,
                        "tb_step",
                        include_bytes!("../../assets/icons/step.svg"),
                        "Adım",
                        None,
                        None,
                        "Diyagramı tek bir düğüm adımı kadar çalıştırıp duraklatır",
                    ) {
                        runner.step_single(_diagram);
                    }

                    ui.add_space(4.0);

                    // Speed Selector: Manual Editable DragValue + Preset Dropdown
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hız:").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                        ui.add(
                            egui::DragValue::new(&mut runner.delay_ms)
                                .suffix(" ms")
                                .range(0..=10000)
                                .speed(5.0),
                        )
                        .on_hover_text("Çalışma adım gecikmesini milisaniye (ms) olarak elle girin veya sürükleyin");

                        ComboBox::from_id_salt("toolbar_speed_selector")
                            .width(50.0)
                            .selected_text("Hızlı Seçim")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut runner.delay_ms, 0, "Anlık (0 ms)");
                                ui.selectable_value(&mut runner.delay_ms, 10, "Çok Hızlı (10 ms)");
                                ui.selectable_value(&mut runner.delay_ms, 100, "Hızlı (100 ms)");
                                ui.selectable_value(&mut runner.delay_ms, 300, "Normal (300 ms)");
                                ui.selectable_value(&mut runner.delay_ms, 500, "Yavaş (500 ms)");
                                ui.selectable_value(&mut runner.delay_ms, 1000, "Çok Yavaş (1000 ms)");
                            });
                    });
                });

                // 4. RIGHT SECTION: VIEW PANELS & THEME TOGGLE
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Theme toggle
                    let theme_icon = if *is_dark { "Koyu Tema" } else { "Açık Tema" };
                    if ui.selectable_label(false, RichText::new(theme_icon).size(12.0)).clicked() {
                        *is_dark = !*is_dark;
                    }

                    ui.separator();

                    // View toggles
                    ui.toggle_value(show_chart, RichText::new("📊 Grafik").size(12.0));
                    ui.toggle_value(show_console, RichText::new("Konsol").size(12.0));
                    ui.toggle_value(show_stack, RichText::new("Çağrı Yığını").size(12.0));
                    ui.toggle_value(show_vars, RichText::new("Değişkenler").size(12.0));
                });
            });
        });

    action
}
