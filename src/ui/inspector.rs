use crate::engine::runner::Runner;
use crate::model::diagram::Diagram;
use crate::model::node::NodeType;
use egui::{RichText, ScrollArea, Ui};

pub fn collect_diagram_variables(diagram: &Diagram) -> Vec<String> {
    let mut vars = Vec::new();

    let mut add_candidate = |candidate: &str| {
        let trimmed = candidate.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with('"')
            && !trimmed.starts_with('\'')
            && trimmed.parse::<f64>().is_err()
            && trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            if !vars.contains(&trimmed.to_string()) {
                vars.push(trimmed.to_string());
            }
        }
    };

    for node in &diagram.nodes {
        match node.node_type {
            NodeType::Start => {
                for v in node.expr1.split(',') {
                    add_candidate(v);
                }
            }
            NodeType::Definition => {
                add_candidate(&node.expr1);
            }
            NodeType::Input => {
                add_candidate(&node.expr2);
            }
            NodeType::Add
            | NodeType::Subtract
            | NodeType::Multiply
            | NodeType::Divide
            | NodeType::Action
            | NodeType::Function => {
                add_candidate(&node.target_var);
                add_candidate(&node.expr1);
                add_candidate(&node.expr2);
            }
            NodeType::IfEqual
            | NodeType::IfGreater
            | NodeType::IfGreaterEqual
            | NodeType::IfLess
            | NodeType::IfLessEqual => {
                add_candidate(&node.expr1);
                add_candidate(&node.expr2);
            }
            _ => {}
        }
    }

    vars.sort();
    vars
}

fn render_var_input_with_dropdown(
    ui: &mut Ui,
    field_str: &mut String,
    known_vars: &[String],
    combo_id: &'static str,
) {
    ui.horizontal(|ui| {
        ui.text_edit_singleline(field_str);

        if !known_vars.is_empty() {
            egui::ComboBox::from_id_salt(combo_id)
                .width(20.0)
                .selected_text("📋")
                .show_ui(ui, |ui| {
                    for var in known_vars {
                        if ui.selectable_label(field_str == var, var).clicked() {
                            *field_str = var.clone();
                        }
                    }
                });
        }
    });
}

pub fn render_inspector(ui: &mut Ui, diagram: &mut Diagram, selected_node_id: Option<&str>) {
    let known_vars = collect_diagram_variables(diagram);

    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.heading(RichText::new("Özellikler").size(13.0).strong());
        ui.separator();

        let node_id = match selected_node_id {
            Some(id) => id,
            None => {
                ui.label(RichText::new("Düzenlemek için canvas üzerinde bir nesne seçin.").weak());
                return;
            }
        };

        let node = match diagram.get_node_mut(node_id) {
            Some(n) => n,
            None => return,
        };

        ui.horizontal(|ui| {
            ui.label("ID:");
            ui.monospace(&node.id);
        });

        ui.horizontal(|ui| {
            ui.label("Tür:");
            ui.label(RichText::new(node.node_type.display_name()).strong());
        });

        ui.separator();

        match node.node_type {
            NodeType::Start => {
                ui.label("Başlangıç Değişkenleri:");
                ui.text_edit_singleline(&mut node.expr1);
            }
            NodeType::Definition => {
                ui.label("Değişken Adı:");
                render_var_input_with_dropdown(ui, &mut node.expr1, &known_vars, "inspect_def_var");
                ui.label("Değer / İfade:");
                render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_def_val");
            }
            NodeType::Input => {
                ui.label("İstem Mesajı:");
                ui.text_edit_singleline(&mut node.expr1);
                ui.label("Hedef Değişken:");
                render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_in_target");
            }
            NodeType::Output => {
                ui.label("Başlık / Etiket:");
                ui.text_edit_singleline(&mut node.expr1);
                ui.label("Değer / İfade:");
                render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_out_val");
            }
            NodeType::IfEqual
            | NodeType::IfGreater
            | NodeType::IfGreaterEqual
            | NodeType::IfLess
            | NodeType::IfLessEqual => {
                ui.label("Değişken 1:");
                render_var_input_with_dropdown(ui, &mut node.expr1, &known_vars, "inspect_if_v1");
                ui.label("Değer / Değişken 2:");
                render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_if_v2");
            }
            NodeType::Add
            | NodeType::Subtract
            | NodeType::Multiply
            | NodeType::Divide => {
                ui.label("Girdi 1:");
                render_var_input_with_dropdown(ui, &mut node.expr1, &known_vars, "inspect_math_v1");
                ui.label("Girdi 2:");
                render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_math_v2");
                ui.label("Hedef Değişken:");
                render_var_input_with_dropdown(ui, &mut node.target_var, &known_vars, "inspect_math_target");
            }
            NodeType::Action => {
                ui.label("İşlem Etiketi:");
                ui.text_edit_singleline(&mut node.label);

                if node.label.trim().eq_ignore_ascii_case("JOIN") {
                    ui.label("Parça 1:");
                    render_var_input_with_dropdown(ui, &mut node.expr1, &known_vars, "inspect_join_v1");
                    ui.label("Parça 2:");
                    render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_join_v2");
                    ui.label("Hedef Değişken:");
                    render_var_input_with_dropdown(ui, &mut node.target_var, &known_vars, "inspect_join_target");
                } else if node.label.trim().eq_ignore_ascii_case("COMP") {
                    ui.label("Karşılaştırılan 1:");
                    render_var_input_with_dropdown(ui, &mut node.expr1, &known_vars, "inspect_comp_v1");
                    ui.label("Karşılaştırılan 2:");
                    render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_comp_v2");
                    ui.label("Hedef Değişken:");
                    render_var_input_with_dropdown(ui, &mut node.target_var, &known_vars, "inspect_comp_target");
                } else {
                    ui.label("Hedef Değişken:");
                    render_var_input_with_dropdown(ui, &mut node.target_var, &known_vars, "inspect_act_target");
                    ui.label("Değer / İfade:");
                    render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_act_val");
                }
            }
            NodeType::Function => {
                ui.label("Fonksiyon Adı:");
                ui.text_edit_singleline(&mut node.expr1);
                ui.label("Girdi İfadesi:");
                render_var_input_with_dropdown(ui, &mut node.expr2, &known_vars, "inspect_fn_arg");
                ui.label("Hedef Değişken:");
                render_var_input_with_dropdown(ui, &mut node.target_var, &known_vars, "inspect_fn_target");
            }
            _ => {}
        }

        ui.separator();
        ui.label("Boyutlar:");
        ui.horizontal(|ui| {
            ui.label("G:");
            ui.add(egui::DragValue::new(&mut node.width).range(20.0..=400.0));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut node.height).range(20.0..=400.0));
        });
    });
}

pub fn render_variables_and_stack(ui: &mut Ui, runner: &Runner, show_vars: bool, show_stack: bool) {
    if !show_vars && !show_stack {
        return;
    }

    ui.add_space(8.0);
    ui.separator();

    if show_vars {
        ui.heading(RichText::new("Değişkenler").size(13.0).strong());
        ui.add_space(4.0);

        if runner.variables.is_empty() {
            ui.label(RichText::new("Henüz tanımlı değişken yok.").weak());
        } else {
            ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    egui::Grid::new("vars_docked_grid")
                        .striped(true)
                        .min_col_width(110.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Değişken").strong());
                            ui.label(RichText::new("Değer").strong());
                            ui.end_row();

                            for (k, v) in &runner.variables {
                                ui.monospace(k);
                                ui.monospace(v.to_string_val());
                                ui.end_row();
                            }
                        });
                });
        }
    }

    if show_stack {
        ui.add_space(6.0);
        ui.heading(RichText::new("Çağrı Yığını").size(13.0).strong());
        ui.add_space(4.0);

        if runner.diagram_stack.is_empty() {
            ui.label(RichText::new("Yığın boş (Ana Akış)").weak());
        } else {
            for frame in &runner.diagram_stack {
                ui.label(format!("↳ {} (Dönüş: {})", frame.function_name, frame.return_node_id));
            }
        }
    }
}
