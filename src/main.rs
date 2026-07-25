mod engine;
mod model;
mod ui;

use engine::runner::{ExecutionState, Runner};
use model::diagram::Diagram;
use ui::canvas::{render_canvas, CanvasState};
use ui::inspector::{render_inspector, render_variables_and_stack};
use ui::modals::{render_input_modal, ModalInputState};
use ui::palette::{render_palette, PaletteState};
use ui::toolbar::render_toolbar;

use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct DeerApp {
    diagram: Diagram,
    runner: Runner,
    canvas_state: CanvasState,
    palette_state: PaletteState,
    modal_input: ModalInputState,
    chart_state: ui::chart::ChartWindowState,
    show_vars: bool,
    show_stack: bool,
    show_console: bool,
    is_dark: bool,
    last_step_time: Instant,
    current_filepath: Option<PathBuf>,
    logo_texture: Option<egui::TextureHandle>,
}

impl Default for DeerApp {
    fn default() -> Self {
        let mut diagram = Diagram::new("DEER Diagram");
        let mut current_filepath = None;

        // Try loading sample file examples/B4.fpp if present
        let sample_path = PathBuf::from("examples/B4.fpp");
        if sample_path.exists() {
            if let Ok(content) = fs::read_to_string(&sample_path) {
                if let Ok(loaded) = Diagram::parse_fpp(&content) {
                    diagram = loaded;
                    current_filepath = Some(sample_path);
                }
            }
        }

        Self {
            diagram,
            runner: Runner::new(),
            canvas_state: CanvasState::default(),
            palette_state: PaletteState::default(),
            modal_input: ModalInputState::default(),
            chart_state: ui::chart::ChartWindowState::default(),
            show_vars: true,
            show_stack: false,
            show_console: true,
            is_dark: true,
            last_step_time: Instant::now(),
            current_filepath,
            logo_texture: None,
        }
    }
}

fn apply_deer_theme(ctx: &egui::Context, is_dark: bool) {
    if is_dark {
        let mut visuals = egui::Visuals::dark();
        // Match DEER monochromatic logo palette: #1C2026 Dark Slate, #232830 Panel Slate
        visuals.panel_fill = egui::Color32::from_rgb(28, 32, 38);
        visuals.window_fill = egui::Color32::from_rgb(35, 40, 48);
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 40, 48);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(43, 50, 61);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(58, 67, 81);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(72, 83, 100);

        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240));
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 195, 210));
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

        visuals.selection.bg_fill = egui::Color32::from_rgb(55, 75, 100);
        visuals.selection.stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(138, 150, 164));
        ctx.set_visuals(visuals);
    } else {
        ctx.set_visuals(egui::Visuals::light());
    }
}

fn load_app_icon() -> Option<Arc<egui::IconData>> {
    // Load vector SVG logo directly for macOS Dock & window icon
    let svg_bytes = include_bytes!("../assets/logo.svg");
    if let Ok(color_image) = egui_extras::image::load_svg_bytes(svg_bytes) {
        let width = color_image.width() as u32;
        let height = color_image.height() as u32;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for pixel in &color_image.pixels {
            rgba.push(pixel.r());
            rgba.push(pixel.g());
            rgba.push(pixel.b());
            rgba.push(pixel.a());
        }
        return Some(Arc::new(egui::IconData {
            rgba,
            width,
            height,
        }));
    }
    None
}

impl eframe::App for DeerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply custom DEER monochromatic dark slate theme
        apply_deer_theme(ctx, self.is_dark);
        self.canvas_state.is_dark = self.is_dark;

        // Lazy load official brand logo texture directly from vector logo.svg
        if self.logo_texture.is_none() {
            egui_extras::install_image_loaders(ctx);
            let svg_bytes = include_bytes!("../assets/logo.svg");
            if let Ok(color_image) = egui_extras::image::load_svg_bytes(svg_bytes) {
                let tex_options = egui::TextureOptions {
                    magnification: egui::TextureFilter::Linear,
                    minification: egui::TextureFilter::Linear,
                    ..Default::default()
                };
                self.logo_texture = Some(ctx.load_texture("deer_logo_svg", color_image, tex_options));
            }
        }

        // Auto-step execution when Running
        if self.runner.state == ExecutionState::Running {
            if self.runner.delay_ms == 0 {
                // High-performance loop batch execution (up to 500 steps per frame burst)
                let start_batch = Instant::now();
                let mut batch_count = 0;

                while self.runner.state == ExecutionState::Running
                    && batch_count < 500
                    && start_batch.elapsed() < Duration::from_millis(16)
                {
                    self.runner.step(&self.diagram);
                    batch_count += 1;
                }
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(Duration::from_millis(self.runner.delay_ms));
                if self.last_step_time.elapsed() >= Duration::from_millis(self.runner.delay_ms) {
                    self.runner.step(&self.diagram);
                    self.last_step_time = Instant::now();
                }
            }
        }

        // 1. Top Toolbar Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            let toolbar_action = render_toolbar(
                ui,
                &self.diagram,
                &mut self.runner,
                &mut self.show_vars,
                &mut self.show_stack,
                &mut self.show_console,
                &mut self.chart_state.is_open,
                &mut self.is_dark,
                self.current_filepath.as_deref(),
                self.logo_texture.as_ref(),
            );

            if toolbar_action.new_diagram {
                self.diagram = Diagram::new("New Diagram");
                self.runner.reset();
                self.current_filepath = None;
                self.canvas_state.needs_centering = true;
            }

            if toolbar_action.open_file {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("DEER Diagram (*.dfpp, *.json, *.fpp)", &["dfpp", "json", "fpp"])
                    .pick_file()
                {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(parsed) = Diagram::parse(&content) {
                            self.diagram = parsed;
                            self.runner.reset();
                            self.current_filepath = Some(path);
                            self.canvas_state.needs_centering = true;
                        }
                    }
                }
            }

            if toolbar_action.center_view {
                self.canvas_state.needs_centering = true;
            }

            if toolbar_action.save_file || toolbar_action.save_file_as {
                let target_path = if toolbar_action.save_file_as {
                    rfd::FileDialog::new()
                        .set_file_name("diagram.dfpp")
                        .add_filter("DEER Diagram (*.dfpp)", &["dfpp"])
                        .add_filter("Legacy FlowChart (*.fpp)", &["fpp"])
                        .save_file()
                } else {
                    self.current_filepath.clone().or_else(|| {
                        rfd::FileDialog::new()
                            .set_file_name("diagram.dfpp")
                            .add_filter("DEER Diagram (*.dfpp)", &["dfpp"])
                            .add_filter("Legacy FlowChart (*.fpp)", &["fpp"])
                            .save_file()
                    })
                };

                if let Some(path) = target_path {
                    let exported = if path.extension().and_then(|ext| ext.to_str()) == Some("fpp") {
                        self.diagram.export_fpp()
                    } else {
                        self.diagram.export_json().unwrap_or_else(|_| self.diagram.export_fpp())
                    };

                    if fs::write(&path, exported).is_ok() {
                        self.current_filepath = Some(path);
                    }
                }
            }
        });

        // 2. Bottom Console & Output Panel (Docked IDE Terminal style)
        if self.show_console {
            ui::console::render_console(ctx, &mut self.runner, &mut self.chart_state.is_open);
        }

        // 3. Left Palette Panel (Resizable IDE tool palette)
        egui::SidePanel::left("left_palette")
            .resizable(true)
            .default_width(190.0)
            .width_range(160.0..=300.0)
            .show(ctx, |ui| {
                render_palette(
                    ui,
                    &mut self.diagram,
                    &mut self.runner,
                    &mut self.canvas_state,
                    &mut self.palette_state,
                );
            });

        // 4. Right Panel (Properties Inspector + Variables & Call Stack)
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_inspector(
                        ui,
                        &mut self.diagram,
                        self.canvas_state.selected_node_id.as_deref(),
                    );
                    render_variables_and_stack(ui, &self.runner, self.show_vars, self.show_stack);
                });
            });

        // 5. Central Canvas Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            render_canvas(
                ui,
                &mut self.diagram,
                &self.runner,
                &mut self.canvas_state,
            );
        });

        // 6. Interactive Input Dialog Modal (when waiting for user input during execution)
        render_input_modal(
            ctx,
            &self.diagram,
            &mut self.runner,
            &mut self.modal_input,
        );

        // 7. Interactive Algorithm Result Chart Window
        ui::chart::render_chart_window(ctx, &self.runner, &mut self.chart_state);
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_title("DEER — Diagram Execution Engine in Rust")
        .with_inner_size([1440.0, 920.0])
        .with_min_inner_size([960.0, 640.0]);

    if let Some(icon) = load_app_icon() {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "DEER — Diagram Execution Engine in Rust",
        options,
        Box::new(|_cc| Ok(Box::new(DeerApp::default()))),
    )
}
