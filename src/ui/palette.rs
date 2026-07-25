use super::canvas::{CanvasState, CanvasTool};
use crate::engine::runner::Runner;
use crate::model::diagram::Diagram;
use crate::model::node::NodeType;
use egui::{
    vec2, Align, Color32, Frame, Image, Layout, Margin, RichText, ScrollArea, Stroke, TextEdit, Ui,
};

use std::collections::HashMap;
use egui::TextureHandle;

#[derive(Default)]
pub struct PaletteState {
    pub search_query: String,
    pub texture_cache: HashMap<&'static str, TextureHandle>,
}

pub struct PaletteTheme {
    pub card_fill: Color32,
    pub card_stroke: Color32,
    pub btn_fill: Color32,
    pub btn_stroke: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub search_fill: Color32,
    pub search_stroke: Color32,
}

impl PaletteTheme {
    pub fn new(is_dark: bool) -> Self {
        if is_dark {
            Self {
                card_fill: Color32::from_rgb(23, 28, 36),
                card_stroke: Color32::from_rgb(38, 45, 56),
                btn_fill: Color32::from_rgb(30, 36, 46),
                btn_stroke: Color32::from_rgb(45, 53, 66),
                text_primary: Color32::from_rgb(226, 232, 240),
                text_secondary: Color32::from_rgb(148, 163, 184),
                search_fill: Color32::from_rgb(23, 28, 36),
                search_stroke: Color32::from_rgb(45, 53, 66),
            }
        } else {
            Self {
                card_fill: Color32::from_rgb(241, 245, 249),
                card_stroke: Color32::from_rgb(226, 232, 240),
                btn_fill: Color32::from_rgb(255, 255, 255),
                btn_stroke: Color32::from_rgb(203, 213, 225),
                text_primary: Color32::from_rgb(15, 23, 42),
                text_secondary: Color32::from_rgb(71, 85, 105),
                search_fill: Color32::from_rgb(255, 255, 255),
                search_stroke: Color32::from_rgb(203, 213, 225),
            }
        }
    }
}

fn get_icon_texture(
    ctx: &egui::Context,
    cache: &mut HashMap<&'static str, TextureHandle>,
    key: &'static str,
    svg_bytes: &'static [u8],
) -> Option<TextureHandle> {
    if let Some(tex) = cache.get(key) {
        return Some(tex.clone());
    }
    if let Ok(color_img) = egui_extras::image::load_svg_bytes(svg_bytes) {
        let tex = ctx.load_texture(key, color_img, egui::TextureOptions::LINEAR);
        cache.insert(key, tex.clone());
        return Some(tex);
    }
    None
}

/// Helper: Render a section header badge with SVG vector icon & title
fn render_section_header(
    ui: &mut Ui,
    palette_state: &mut PaletteState,
    icon_key: &'static str,
    svg_bytes: &'static [u8],
    title: &str,
    _accent_color: Color32,
    theme: &PaletteTheme,
) {
    ui.horizontal(|ui| {
        if let Some(tex) = get_icon_texture(ui.ctx(), &mut palette_state.texture_cache, icon_key, svg_bytes) {
            ui.add(Image::new(&tex).fit_to_exact_size(vec2(14.0, 14.0)));
        }
        ui.label(
            RichText::new(title)
                .size(10.5)
                .strong()
                .color(theme.text_secondary),
        );
    });
    ui.add_space(3.0);
}

/// Helper: Left-aligned modern button with SVG vector icon and custom accent tinting
fn render_mode_button(
    ui: &mut Ui,
    palette_state: &mut PaletteState,
    icon_key: &'static str,
    svg_bytes: Option<&'static [u8]>,
    label: &str,
    is_active: bool,
    active_color: Color32,
    tooltip: &str,
    theme: &PaletteTheme,
) -> bool {
    let width = ui.available_width();
    let bg_fill = if is_active {
        active_color.linear_multiply(0.2)
    } else {
        theme.btn_fill
    };
    let stroke_color = if is_active {
        active_color
    } else {
        theme.btn_stroke
    };

    let text_color = if is_active {
        if active_color == Color32::from_rgb(251, 191, 36) {
            Color32::BLACK
        } else {
            active_color
        }
    } else {
        theme.text_primary
    };

    let btn_id = ui.make_persistent_id(label);
    let frame_res = Frame::none()
        .fill(bg_fill)
        .stroke(Stroke::new(1.0, stroke_color))
        .rounding(6.0)
        .inner_margin(Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.set_width(width - 16.0);
            ui.horizontal(|ui| {
                if let Some(bytes) = svg_bytes {
                    if let Some(tex) = get_icon_texture(ui.ctx(), &mut palette_state.texture_cache, icon_key, bytes) {
                        ui.add(egui::Image::new(&tex).fit_to_exact_size(vec2(15.0, 15.0)));
                    }
                }
                ui.label(RichText::new(label).size(12.0).strong().color(text_color));
            });
        });

    let response = ui.interact(frame_res.response.rect, btn_id, egui::Sense::click());
    response.on_hover_text(tooltip).clicked()
}

/// Helper: Left-aligned node creation button supporting SVG vector icons and Drag & Drop
fn render_node_button(
    ui: &mut Ui,
    palette_state: &mut PaletteState,
    icon_key: &'static str,
    svg_bytes: Option<&'static [u8]>,
    label: &str,
    node_type: NodeType,
    accent_color: Color32,
    query: &str,
    desc: &str,
    state: &mut CanvasState,
    theme: &PaletteTheme,
) -> bool {
    if !query.is_empty()
        && !label.to_lowercase().contains(query)
        && !desc.to_lowercase().contains(query)
    {
        return false;
    }

    let width = ui.available_width();
    let btn_id = ui.make_persistent_id(label);

    let frame_res = Frame::none()
        .fill(theme.btn_fill)
        .stroke(Stroke::new(1.0, theme.btn_stroke))
        .rounding(6.0)
        .inner_margin(Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.set_width(width - 16.0);
            ui.horizontal(|ui| {
                if let Some(bytes) = svg_bytes {
                    if let Some(tex) = get_icon_texture(ui.ctx(), &mut palette_state.texture_cache, icon_key, bytes) {
                        ui.add(egui::Image::new(&tex).fit_to_exact_size(vec2(15.0, 15.0)));
                    }
                }
                ui.label(RichText::new(label).size(12.0).strong().color(theme.text_primary));
            });
        });

    let rect = frame_res.response.rect;
    // Draw left accent bar on button edge
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.min, vec2(3.5, rect.height())),
        2.0,
        accent_color,
    );

    let response = ui.interact(rect, btn_id, egui::Sense::click_and_drag());

    // Support Drag and Drop onto Canvas
    if response.drag_started() {
        state.dragging_node_type = Some(node_type);
    }

    response.on_hover_text(desc).clicked()
}

fn add_node_to_diagram(
    diagram: &mut Diagram,
    runner: &mut Runner,
    state: &mut CanvasState,
    node_type: NodeType,
) {
    let center_x = 260.0 - state.pan_offset.x / state.zoom;
    let center_y = 200.0 - state.pan_offset.y / state.zoom;
    let new_id = diagram.add_node(node_type, center_x, center_y);
    runner.rebuild_node_map();
    state.selected_node_id = Some(new_id);
    state.tool = CanvasTool::Select;
}

pub fn render_palette(
    ui: &mut Ui,
    diagram: &mut Diagram,
    runner: &mut Runner,
    state: &mut CanvasState,
    palette_state: &mut PaletteState,
) {
    let theme = PaletteTheme::new(state.is_dark);

    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
        ui.add_space(6.0);

        // Header Title matching DEER logo theme
        ui.horizontal(|ui| {
            if let Some(tex) = get_icon_texture(ui.ctx(), &mut palette_state.texture_cache, "hdr_tools", include_bytes!("../../assets/icons/mode.svg")) {
                ui.add(Image::new(&tex).fit_to_exact_size(vec2(16.0, 16.0)));
            }
            ui.label(
                RichText::new("ARAÇLAR & DÜĞÜMLER")
                    .size(12.0)
                    .strong()
                    .color(theme.text_primary),
            );
        });
        ui.add_space(6.0);

        // Search Filter Input Box
        Frame::none()
            .fill(theme.search_fill)
            .stroke(Stroke::new(1.0, theme.search_stroke))
            .rounding(6.0)
            .inner_margin(Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.add(
                    TextEdit::singleline(&mut palette_state.search_query)
                        .hint_text("Düğüm ara...")
                        .text_color(theme.text_primary)
                        .frame(false)
                        .desired_width(ui.available_width()),
                );
            });

        ui.add_space(8.0);

        let query = palette_state.search_query.trim().to_lowercase();

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    // 1. CHIP / MODE SELECTOR GROUP
                    Frame::none()
                        .fill(theme.card_fill)
                        .stroke(Stroke::new(1.0, theme.card_stroke))
                        .rounding(8.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            render_section_header(ui, palette_state, "hdr_mode", include_bytes!("../../assets/icons/mode.svg"), "ÇALIŞMA MODU", Color32::from_rgb(45, 212, 191), &theme);

                            if render_mode_button(
                                ui,
                                palette_state,
                                "tool_select",
                                Some(include_bytes!("../../assets/icons/select.svg")),
                                "Seç & Taşı",
                                state.tool == CanvasTool::Select,
                                Color32::from_rgb(45, 212, 191),
                                "Canvas üzerindeki nesneleri seçin ve taşıyın",
                                &theme,
                            ) {
                                state.tool = CanvasTool::Select;
                                state.connecting_from_id = None;
                            }
                            ui.add_space(4.0);

                            if render_mode_button(
                                ui,
                                palette_state,
                                "tool_connect",
                                Some(include_bytes!("../../assets/icons/connect.svg")),
                                "Ok Çizgisi Bağla",
                                state.tool == CanvasTool::Connect,
                                Color32::from_rgb(125, 211, 252),
                                "Düğümleri birbirine ok çizgisi ile bağlayın",
                                &theme,
                            ) {
                                state.tool = CanvasTool::Connect;
                            }
                            ui.add_space(4.0);

                            if render_mode_button(
                                ui,
                                palette_state,
                                "tool_delete_node",
                                Some(include_bytes!("../../assets/icons/delete_node.svg")),
                                "Nesne Sil",
                                state.tool == CanvasTool::DeleteNode,
                                Color32::from_rgb(248, 113, 113),
                                "Tıklanan nesneyi siler",
                                &theme,
                            ) {
                                state.tool = CanvasTool::DeleteNode;
                            }
                            ui.add_space(4.0);

                            if render_mode_button(
                                ui,
                                palette_state,
                                "tool_delete_line",
                                Some(include_bytes!("../../assets/icons/delete_line.svg")),
                                "Çizgi Sil",
                                state.tool == CanvasTool::DeleteLine,
                                Color32::from_rgb(248, 113, 113),
                                "Tıklanan bağlantı çizgisini siler",
                                &theme,
                            ) {
                                state.tool = CanvasTool::DeleteLine;
                            }
                        });

                    ui.add_space(8.0);

                    // 2. FLOW CONTROL
                    Frame::none()
                        .fill(theme.card_fill)
                        .stroke(Stroke::new(1.0, theme.card_stroke))
                        .rounding(8.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            render_section_header(ui, palette_state, "hdr_flow", include_bytes!("../../assets/icons/flow.svg"), "AKIŞ KONTROLÜ", Color32::from_rgb(52, 211, 153), &theme);

                            if render_node_button(ui, palette_state, "node_start", Some(include_bytes!("../../assets/icons/start.svg")), "START", NodeType::Start, Color32::from_rgb(52, 211, 153), &query, "Oval - Program Başlangıcı", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Start);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_stop", Some(include_bytes!("../../assets/icons/stop.svg")), "STOP", NodeType::Stop, Color32::from_rgb(248, 113, 113), &query, "Oval - Program Bitişi", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Stop);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_join", Some(include_bytes!("../../assets/icons/join.svg")), "Kesişim (Join)", NodeType::Intersection, Color32::from_rgb(203, 213, 225), &query, "Daire - Akış Birleştirme Noktası", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Intersection);
                            }
                        });

                    ui.add_space(8.0);

                    // 3. CONDITIONS (IF)
                    Frame::none()
                        .fill(theme.card_fill)
                        .stroke(Stroke::new(1.0, theme.card_stroke))
                        .rounding(8.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            render_section_header(ui, palette_state, "hdr_cond", include_bytes!("../../assets/icons/condition.svg"), "KOŞULLAR (IF)", Color32::from_rgb(251, 191, 36), &theme);

                            if render_node_button(ui, palette_state, "node_if_eq", Some(include_bytes!("../../assets/icons/if_equal.svg")), "IF (== Eşit)", NodeType::IfEqual, Color32::from_rgb(251, 191, 36), &query, "Baklava - Eşitlik Koşulu", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::IfEqual);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_if_gt", Some(include_bytes!("../../assets/icons/if_greater.svg")), "IF (> Büyük)", NodeType::IfGreater, Color32::from_rgb(251, 191, 36), &query, "Baklava - Büyüktür Koşulu", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::IfGreater);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_if_gte", Some(include_bytes!("../../assets/icons/if_greater.svg")), "IF (>= B-Eşit)", NodeType::IfGreaterEqual, Color32::from_rgb(251, 191, 36), &query, "Baklava - Büyük-Eşit Koşulu", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::IfGreaterEqual);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_if_lt", Some(include_bytes!("../../assets/icons/if_less.svg")), "IF (< Küçük)", NodeType::IfLess, Color32::from_rgb(251, 191, 36), &query, "Baklava - Küçüktür Koşulu", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::IfLess);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_if_lte", Some(include_bytes!("../../assets/icons/if_less.svg")), "IF (<= K-Eşit)", NodeType::IfLessEqual, Color32::from_rgb(251, 191, 36), &query, "Baklava - Küçük-Eşit Koşulu", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::IfLessEqual);
                            }
                        });

                    ui.add_space(8.0);

                    // 4. MATH & DEFINITIONS
                    Frame::none()
                        .fill(theme.card_fill)
                        .stroke(Stroke::new(1.0, theme.card_stroke))
                        .rounding(8.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            render_section_header(ui, palette_state, "hdr_math", include_bytes!("../../assets/icons/math.svg"), "ARİTMETİK & TANIM", Color32::from_rgb(192, 132, 252), &theme);

                            if render_node_button(ui, palette_state, "node_def", Some(include_bytes!("../../assets/icons/definition.svg")), "DEFINITION (=)", NodeType::Definition, Color32::from_rgb(125, 211, 252), &query, "Değişken Tanımlama", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Definition);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_add", Some(include_bytes!("../../assets/icons/add.svg")), "ADD (+ Topla)", NodeType::Add, Color32::from_rgb(192, 132, 252), &query, "Toplama İşlemi", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Add);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_sub", Some(include_bytes!("../../assets/icons/subtract.svg")), "SUBTRACT (- Çıkar)", NodeType::Subtract, Color32::from_rgb(192, 132, 252), &query, "Çıkarma İşlemi", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Subtract);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_mul", Some(include_bytes!("../../assets/icons/multiply.svg")), "MULTIPLY (* Çarp)", NodeType::Multiply, Color32::from_rgb(192, 132, 252), &query, "Çarpma İşlemi", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Multiply);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_div", Some(include_bytes!("../../assets/icons/divide.svg")), "DIVIDE (/ Böl)", NodeType::Divide, Color32::from_rgb(192, 132, 252), &query, "Bölme İşlemi", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Divide);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_act", Some(include_bytes!("../../assets/icons/action.svg")), "ACTION (Atama)", NodeType::Action, Color32::from_rgb(125, 211, 252), &query, "Genel İşlem & Atama", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Action);
                            }
                        });

                    ui.add_space(8.0);

                    // 5. INPUT & OUTPUT
                    Frame::none()
                        .fill(theme.card_fill)
                        .stroke(Stroke::new(1.0, theme.card_stroke))
                        .rounding(8.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            render_section_header(ui, palette_state, "hdr_io", include_bytes!("../../assets/icons/io.svg"), "GİRDİ & ÇIKTI", Color32::from_rgb(45, 212, 191), &theme);

                            if render_node_button(ui, palette_state, "node_in", Some(include_bytes!("../../assets/icons/input.svg")), "INPUT (Giriş)", NodeType::Input, Color32::from_rgb(45, 212, 191), &query, "Kullanıcıdan Değer Alma", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Input);
                            }
                            ui.add_space(3.0);

                            if render_node_button(ui, palette_state, "node_out", Some(include_bytes!("../../assets/icons/output.svg")), "OUTPUT (Çıkış)", NodeType::Output, Color32::from_rgb(251, 146, 60), &query, "Ekrana / Konsola Yazdırma", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Output);
                            }
                        });

                    ui.add_space(8.0);

                    // 6. FUNCTIONS & WINDOWS
                    Frame::none()
                        .fill(theme.card_fill)
                        .stroke(Stroke::new(1.0, theme.card_stroke))
                        .rounding(8.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            render_section_header(ui, palette_state, "hdr_func", include_bytes!("../../assets/icons/function.svg"), "FONKSİYONLAR", Color32::from_rgb(232, 121, 249), &theme);

                            if render_node_button(ui, palette_state, "node_func", Some(include_bytes!("../../assets/icons/function.svg")), "FUNCTION (Çağrı)", NodeType::Function, Color32::from_rgb(232, 121, 249), &query, "Alt-Diyagram veya Dahili Fonksiyon Çağrısı", state, &theme) {
                                add_node_to_diagram(diagram, runner, state, NodeType::Function);
                            }
                        });

                    ui.add_space(8.0);

                    // 7. SAMPLES (Embedded Directly)
                    Frame::none()
                        .fill(theme.card_fill)
                        .stroke(Stroke::new(1.0, theme.card_stroke))
                        .rounding(8.0)
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            render_section_header(ui, palette_state, "hdr_sample", include_bytes!("../../assets/icons/sample.svg"), "ÖRNEK DİYAGRAMLAR", Color32::from_rgb(148, 163, 184), &theme);

                            let samples: &[(&'static str, &'static str, &'static str, &[u8])] = &[
                                ("sample_b4", "B4 (Atama / Değişken)", "B4.fpp", include_bytes!("../../examples/B4.fpp")),
                                ("sample_a", "A (Temel Akış)", "A.fpp", include_bytes!("../../examples/A.fpp")),
                                ("sample_t2", "T2 (Döngü & Şart)", "T2.fpp", include_bytes!("../../examples/T2.fpp")),
                                ("sample_g", "G (Grafik & Hesap)", "G.fpp", include_bytes!("../../examples/G.fpp")),
                            ];

                            let width = ui.available_width();
                            let sample_svg = include_bytes!("../../assets/icons/sample.svg");
                            for (key, label, _file_name, data) in samples {
                                let btn_id = ui.make_persistent_id(label);
                                let frame_res = Frame::none()
                                    .fill(theme.btn_fill)
                                    .stroke(Stroke::new(1.0, theme.btn_stroke))
                                    .rounding(6.0)
                                    .inner_margin(Margin::symmetric(8.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.set_width(width - 16.0);
                                        ui.horizontal(|ui| {
                                            if let Some(tex) = get_icon_texture(ui.ctx(), &mut palette_state.texture_cache, key, sample_svg) {
                                                ui.add(egui::Image::new(&tex).fit_to_exact_size(vec2(15.0, 15.0)));
                                            }
                                            ui.label(RichText::new(*label).size(11.5).strong().color(theme.text_primary));
                                        });
                                    });

                                let response = ui.interact(frame_res.response.rect, btn_id, egui::Sense::click());
                                if response.clicked() {
                                    if let Ok(content) = std::str::from_utf8(data) {
                                        if let Ok(loaded) = Diagram::parse_fpp(content) {
                                            *diagram = loaded;
                                            runner.reset();
                                            state.needs_centering = true;
                                        }
                                    }
                                }
                                ui.add_space(3.0);
                            }
                        });

                    ui.add_space(14.0);
                    ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                        ui.label(
                            RichText::new("DEER v0.1.0 — Rust Engine")
                                .size(10.0)
                                .color(theme.text_secondary),
                        );
                    });
                });
            });
    });
}
