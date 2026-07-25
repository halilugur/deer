use crate::engine::runner::{ExecutionState, Runner};
use crate::model::diagram::Diagram;
use egui::{Context, RichText, Window};

#[derive(Default)]
pub struct ModalInputState {
    pub text_input: String,
    pub focus_requested: bool,
}

pub fn render_input_modal(
    ctx: &Context,
    _diagram: &Diagram,
    runner: &mut Runner,
    input_state: &mut ModalInputState,
) {
    let mut submit_action: Option<(String, String, String, bool)> = None;

    if let ExecutionState::WaitingForInput {
        prompt,
        target_var,
        next_node_id,
    } = &runner.state.clone()
    {
        Window::new("Giriş İsteniyor")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(prompt).strong().size(14.0));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("`{}` =", target_var));
                        let re = ui.text_edit_singleline(&mut runner.input_text);
                        input_state.text_input = runner.input_text.clone();

                        // Request focus only once when dialog appears
                        if !input_state.focus_requested {
                            re.request_focus();
                            input_state.focus_requested = true;
                        }
                    });
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Tamam").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            submit_action = Some((
                                runner.input_text.clone(),
                                target_var.clone(),
                                next_node_id.clone(),
                                false,
                            ));
                        }
                        ui.add_space(6.0);
                        if ui.button(RichText::new("Adım").strong()).clicked() {
                            submit_action = Some((
                                runner.input_text.clone(),
                                target_var.clone(),
                                next_node_id.clone(),
                                true,
                            ));
                        }
                    });
                    ui.add_space(4.0);
                });
            });
    } else {
        input_state.focus_requested = false;
    }

    if let Some((val_str, target_var, next_id, force_step)) = submit_action {
        if force_step {
            runner.step_mode = true;
        }
        runner.submit_input(&val_str, &target_var, &next_id);
        input_state.text_input.clear();
        input_state.focus_requested = false;
    }
}
