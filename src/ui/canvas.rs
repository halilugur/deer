use crate::engine::runner::Runner;
use crate::model::connector::BranchCondition;
use crate::model::diagram::Diagram;
use crate::model::node::{Node, NodeType};
use egui::epaint::PathShape;
use egui::{
    pos2, vec2, Align2, Color32, FontId, PointerButton, Pos2, Rect, Response, RichText, Sense,
    Stroke, Ui, Vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasTool {
    Select,
    Connect,
    DeleteNode,
    DeleteLine,
}

pub struct CanvasState {
    pub tool: CanvasTool,
    pub selected_node_id: Option<String>,
    pub selected_line_id: Option<String>,
    pub connecting_from_id: Option<String>,
    pub pan_offset: Vec2,
    pub zoom: f32,
    pub is_dark: bool,
    pub needs_centering: bool,
    pub dragging_node_type: Option<NodeType>,
    pub node_drag_accum: Option<(String, f32, f32, f32, f32)>,
}

fn clip_ray_to_node_shape(center: Pos2, size: Vec2, dir: Vec2, node_type: NodeType) -> Pos2 {
    let half_w = size.x * 0.5;
    let half_h = size.y * 0.5;
    if dir.x == 0.0 && dir.y == 0.0 {
        return center;
    }

    match node_type {
        NodeType::IfEqual
        | NodeType::IfGreater
        | NodeType::IfGreaterEqual
        | NodeType::IfLess
        | NodeType::IfLessEqual => {
            // Diamond geometry equation: |x|/half_w + |y|/half_h = 1
            let denom = (dir.x.abs() / half_w) + (dir.y.abs() / half_h);
            if denom > 0.0 {
                let t = 1.0 / denom;
                center + dir * t
            } else {
                center
            }
        }
        NodeType::Intersection => {
            // Circle geometry: radius = min(half_w, half_h)
            let r = half_w.min(half_h);
            center + dir * r
        }
        NodeType::Input | NodeType::Output => {
            // Parallelogram geometry with horizontal skew
            let skew = 12.0;
            let eff_w = (half_w - skew * 0.5).max(1.0);
            let tx = if dir.x != 0.0 { eff_w / dir.x.abs() } else { f32::INFINITY };
            let ty = if dir.y != 0.0 { half_h / dir.y.abs() } else { f32::INFINITY };
            let t = tx.min(ty);
            center + dir * t
        }
        _ => {
            // Standard Rectangle / Rounded Rectangle geometry
            let tx = if dir.x != 0.0 { half_w / dir.x.abs() } else { f32::INFINITY };
            let ty = if dir.y != 0.0 { half_h / dir.y.abs() } else { f32::INFINITY };
            let t = tx.min(ty);
            center + dir * t
        }
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            tool: CanvasTool::Select,
            selected_node_id: None,
            selected_line_id: None,
            connecting_from_id: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            is_dark: true,
            needs_centering: true,
            dragging_node_type: None,
            node_drag_accum: None,
        }
    }
}

pub fn render_canvas(
    ui: &mut Ui,
    diagram: &mut Diagram,
    runner: &Runner,
    state: &mut CanvasState,
) -> Response {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    let rect = response.rect;

    // Handle Mouse Wheel Zoom & Touchpad Pinch-to-Zoom towards Mouse Pointer
    if response.hovered() {
        let zoom_delta = ui.input(|i| i.zoom_delta());
        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);

        let factor = if zoom_delta != 1.0 {
            zoom_delta
        } else if scroll_y != 0.0 {
            if scroll_y > 0.0 { 1.08 } else { 0.92 }
        } else {
            1.0
        };

        if factor != 1.0 {
            let pointer_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(rect.center());
            let old_zoom = state.zoom;
            let new_zoom = (old_zoom * factor).clamp(0.25, 3.5);

            if (new_zoom - old_zoom).abs() > 0.001 {
                let cursor_offset = pointer_pos - rect.min - state.pan_offset;
                state.pan_offset -= cursor_offset * (new_zoom / old_zoom - 1.0);
                state.zoom = new_zoom;
            }
        }
    }

    // Auto-center diagram inside viewport when requested or on initial open
    if state.needs_centering && !diagram.nodes.is_empty() && rect.width() > 50.0 && rect.height() > 50.0 {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for node in &diagram.nodes {
            min_x = min_x.min(node.x);
            max_x = max_x.max(node.x + node.width);
            min_y = min_y.min(node.y);
            max_y = max_y.max(node.y + node.height);
        }

        let content_center_x = (min_x + max_x) / 2.0;
        let content_center_y = (min_y + max_y) / 2.0;

        let canvas_center_x = rect.width() / 2.0;
        let canvas_center_y = rect.height() / 2.0;

        state.pan_offset = vec2(
            canvas_center_x - (content_center_x * state.zoom),
            canvas_center_y - (content_center_y * state.zoom),
        );

        state.needs_centering = false;
    }

    // Draw background grid
    let bg_color = if state.is_dark {
        Color32::from_rgb(28, 32, 38)
    } else {
        Color32::from_rgb(245, 247, 250)
    };
    painter.rect_filled(rect, 0.0, bg_color);

    // Grid dots
    let grid_size = 20.0 * state.zoom;
    let grid_color = if state.is_dark {
        Color32::from_white_alpha(15)
    } else {
        Color32::from_black_alpha(15)
    };

    let start_x = (rect.min.x + state.pan_offset.x) % grid_size;
    let start_y = (rect.min.y + state.pan_offset.y) % grid_size;

    let mut x = rect.min.x + start_x;
    while x < rect.max.x {
        let mut y = rect.min.y + start_y;
        while y < rect.max.y {
            painter.circle_filled(pos2(x, y), 1.0, grid_color);
            y += grid_size;
        }
        x += grid_size;
    }

    let mut any_item_clicked = false;
    let mut any_item_dragged = false;

    // Transform helper
    let to_screen = |x: f32, y: f32| -> Pos2 {
        pos2(
            rect.min.x + (x * state.zoom) + state.pan_offset.x,
            rect.min.y + (y * state.zoom) + state.pan_offset.y,
        )
    };

    // Draw Connectors first (behind nodes)
    let mut line_to_delete: Option<String> = None;
    for connector in &diagram.connectors {
        let from_node = diagram.get_node(&connector.from_id);
        let to_node = diagram.get_node(&connector.to_id);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let c1 = to_screen(from.x + from.width * 0.5, from.y + from.height * 0.5);
            let s1 = vec2(from.width * state.zoom, from.height * state.zoom);

            let c2 = to_screen(to.x + to.width * 0.5, to.y + to.height * 0.5);
            let s2 = vec2(to.width * state.zoom, to.height * state.zoom);

            let delta = c2 - c1;
            let dist = delta.length();
            let (start, end, dir) = if dist > 1.0 {
                let d = delta / dist;
                let st = clip_ray_to_node_shape(c1, s1, d, from.node_type);
                let en = clip_ray_to_node_shape(c2, s2, -d, to.node_type);
                (st, en, d)
            } else {
                (c1, c2, vec2(0.0, 1.0))
            };

            let is_selected = state.selected_line_id.as_deref() == Some(&connector.id);
            let is_active = runner.current_node_id.as_deref() == Some(&connector.from_id);

            let stroke_color = if is_active {
                Color32::from_rgb(0, 255, 180)
            } else if is_selected {
                Color32::from_rgb(255, 170, 0)
            } else {
                match connector.condition {
                    BranchCondition::Yes => Color32::from_rgb(52, 211, 153),
                    BranchCondition::No => Color32::from_rgb(248, 113, 113),
                    BranchCondition::Default => if state.is_dark { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) },
                }
            };

            let stroke = Stroke::new(if is_active || is_selected { 3.0 } else { 2.0 }, stroke_color);
            painter.line_segment([start, end], stroke);

            // Draw filled arrowhead triangle at target node border
            let arrow_len = (11.0 * state.zoom).clamp(8.0, 16.0);
            let arrow_half_w = (5.0 * state.zoom).clamp(4.0, 9.0);
            let back = end - dir * arrow_len;
            let normal = vec2(-dir.y, dir.x);
            let p1 = back + normal * arrow_half_w;
            let p2 = back - normal * arrow_half_w;
            painter.add(egui::Shape::convex_polygon(vec![end, p1, p2], stroke_color, Stroke::NONE));

            // Branch condition label badge (YES / NO / EVET / HAYIR)
            let mid = start + (end - start) * 0.5;
            let label = match connector.condition {
                BranchCondition::Yes => Some("EVET"),
                BranchCondition::No => Some("HAYIR"),
                BranchCondition::Default => None,
            };

            if let Some(lbl) = label {
                let badge_bg = match connector.condition {
                    BranchCondition::Yes => Color32::from_rgb(16, 185, 129),
                    BranchCondition::No => Color32::from_rgb(239, 68, 68),
                    _ => Color32::BLACK,
                };

                let font_id = FontId::proportional(10.0);
                let galley = painter.layout_no_wrap(lbl.to_string(), font_id, Color32::WHITE);
                let badge_rect = Rect::from_center_size(mid, galley.size() + vec2(8.0, 4.0));
                painter.rect_filled(badge_rect, 3.0, badge_bg);
                painter.galley(badge_rect.min + vec2(4.0, 2.0), galley, Color32::WHITE);
            }

            // Click detection on connector line
            if state.tool == CanvasTool::DeleteLine || state.tool == CanvasTool::Select {
                let dist = dist_to_line(response.interact_pointer_pos(), start, end);
                if response.clicked() && dist < 8.0 {
                    any_item_clicked = true;
                    if state.tool == CanvasTool::DeleteLine {
                        line_to_delete = Some(connector.id.clone());
                    } else {
                        state.selected_line_id = Some(connector.id.clone());
                        state.selected_node_id = None;
                    }
                }
            }
        }
    }

    if let Some(lid) = line_to_delete {
        diagram.delete_connector(&lid);
        state.selected_line_id = None;
    }

const GRID_SNAP_SIZE: f32 = 20.0;

fn snap_to_grid(val: f32) -> f32 {
    (val / GRID_SNAP_SIZE).round() * GRID_SNAP_SIZE
}

    // Clone nodes vector for rendering to prevent borrow conflicts
    let nodes_list = diagram.nodes.clone();
    let mut node_to_delete: Option<String> = None;
    let mut new_connection: Option<(String, String, BranchCondition)> = None;

    // Draw Nodes
    for node in &nodes_list {
        let pos = to_screen(node.x, node.y);
        let size = vec2(node.width * state.zoom, node.height * state.zoom);
        let node_rect = Rect::from_min_size(pos, size);

        let is_selected = state.selected_node_id.as_deref() == Some(&node.id);
        let is_active = runner.current_node_id.as_deref() == Some(&node.id);

        render_node_shape(
            &painter,
            node_rect,
            node,
            is_selected,
            is_active,
            state.is_dark,
        );

        // Node click & drag interaction
        let node_id = ui.make_persistent_id(&node.id);
        let interact = ui.interact(node_rect, node_id, Sense::click_and_drag());

        if interact.clicked() {
            any_item_clicked = true;
            match state.tool {
                CanvasTool::Select => {
                    state.selected_node_id = Some(node.id.clone());
                    state.selected_line_id = None;
                }
                CanvasTool::Connect => {
                    if let Some(from_id) = state.connecting_from_id.take() {
                        if from_id != node.id {
                            let condition = if let Some(from_node) = diagram.get_node(&from_id) {
                                match from_node.node_type {
                                    NodeType::IfEqual
                                    | NodeType::IfGreater
                                    | NodeType::IfGreaterEqual
                                    | NodeType::IfLess
                                    | NodeType::IfLessEqual => BranchCondition::Yes,
                                    _ => BranchCondition::Default,
                                }
                            } else {
                                BranchCondition::Default
                            };
                            new_connection = Some((from_id, node.id.clone(), condition));
                        }
                    } else {
                        state.connecting_from_id = Some(node.id.clone());
                    }
                }
                CanvasTool::DeleteNode => {
                    node_to_delete = Some(node.id.clone());
                }
                _ => {}
            }
        }

        let mut handle_was_dragged = false;

        // Render 4 Corner Mouse Resize Handles for Selected Node
        if is_selected && state.tool == CanvasTool::Select {
            let handle_size = (10.0 * state.zoom).max(8.0);
            let handle_fill = if state.is_dark {
                Color32::from_rgb(251, 191, 36)
            } else {
                Color32::from_rgb(217, 119, 6)
            };
            let handle_stroke = Stroke::new(1.0, Color32::from_rgb(20, 20, 20));

            let corners = [
                (node_rect.max, egui::CursorIcon::ResizeNwSe, "se", false, false), // Bottom-Right
                (pos2(node_rect.min.x, node_rect.max.y), egui::CursorIcon::ResizeNeSw, "sw", true, false), // Bottom-Left
                (pos2(node_rect.max.x, node_rect.min.y), egui::CursorIcon::ResizeNeSw, "ne", false, true), // Top-Right
                (node_rect.min, egui::CursorIcon::ResizeNwSe, "nw", true, true), // Top-Left
            ];

            for (center, cursor, suffix, moves_x, moves_y) in corners {
                let handle_rect = Rect::from_center_size(center, vec2(handle_size, handle_size));
                painter.rect_filled(handle_rect, 2.0, handle_fill);
                painter.rect_stroke(handle_rect, 2.0, handle_stroke);

                let handle_id = ui.make_persistent_id(format!("{}_resize_{}", node.id, suffix));
                let handle_interact = ui.interact(handle_rect, handle_id, Sense::drag());

                if handle_interact.hovered() || handle_interact.dragged() {
                    ui.ctx().set_cursor_icon(cursor);
                }

                if handle_interact.dragged() {
                    handle_was_dragged = true;
                    if let Some(n) = diagram.get_node_mut(&node.id) {
                        let delta = handle_interact.drag_delta() / state.zoom;
                        let min_w = match n.node_type {
                            NodeType::Intersection => 20.0,
                            _ => 40.0,
                        };
                        let min_h = match n.node_type {
                            NodeType::Intersection => 20.0,
                            _ => 25.0,
                        };

                        if moves_x {
                            let new_w = (n.width - delta.x).max(min_w);
                            n.x += n.width - new_w;
                            n.width = new_w;
                        } else {
                            n.width = (n.width + delta.x).max(min_w);
                        }

                        if moves_y {
                            let new_h = (n.height - delta.y).max(min_h);
                            n.y += n.height - new_h;
                            n.height = new_h;
                        } else {
                            n.height = (n.height + delta.y).max(min_h);
                        }
                    }
                }
            }
        }

        if handle_was_dragged || interact.dragged() {
            any_item_dragged = true;
        }

        if interact.drag_started() {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                let center_x = node.x + node.width * 0.5;
                let center_y = node.y + node.height * 0.5;
                state.node_drag_accum = Some((node.id.clone(), center_x, center_y, pointer_pos.x, pointer_pos.y));
            }
        }

        if !handle_was_dragged && interact.dragged() && state.tool == CanvasTool::Select {
            if let Some((ref id, orig_cx, orig_cy, start_px, start_py)) = state.node_drag_accum {
                if id == &node.id {
                    if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let total_drag_x = (pointer_pos.x - start_px) / state.zoom;
                        let total_drag_y = (pointer_pos.y - start_py) / state.zoom;
                        if let Some(n) = diagram.get_node_mut(&node.id) {
                            let target_cx = snap_to_grid(orig_cx + total_drag_x);
                            let target_cy = snap_to_grid(orig_cy + total_drag_y);
                            n.x = target_cx - n.width * 0.5;
                            n.y = target_cy - n.height * 0.5;
                        }
                    }
                }
            } else if let Some(n) = diagram.get_node_mut(&node.id) {
                let target_cx = snap_to_grid(n.x + n.width * 0.5 + interact.drag_delta().x / state.zoom);
                let target_cy = snap_to_grid(n.y + n.height * 0.5 + interact.drag_delta().y / state.zoom);
                n.x = target_cx - n.width * 0.5;
                n.y = target_cy - n.height * 0.5;
            }
        }

        if interact.drag_stopped() {
            state.node_drag_accum = None;
            if let Some(n) = diagram.get_node_mut(&node.id) {
                let center_x = snap_to_grid(n.x + n.width * 0.5);
                let center_y = snap_to_grid(n.y + n.height * 0.5);
                n.x = center_x - n.width * 0.5;
                n.y = center_y - n.height * 0.5;
            }
        }
    }

    if let Some((from, to, cond)) = new_connection {
        diagram.add_connector(&from, &to, cond);
    }

    if let Some(nid) = node_to_delete {
        diagram.delete_node(&nid);
        state.selected_node_id = None;
    }

    // Active line connection preview line when connecting (rendered ON TOP of nodes)
    if let Some(ref from_id) = state.connecting_from_id {
        if let Some(from_node) = diagram.get_node(from_id) {
            let c1 = to_screen(from_node.x + from_node.width * 0.5, from_node.y + from_node.height * 0.5);
            let s1 = vec2(from_node.width * state.zoom, from_node.height * state.zoom);

            if let Some(mouse_pos) = response.hover_pos() {
                let delta = mouse_pos - c1;
                let dist = delta.length();
                let (start, end) = if dist > 1.0 {
                    let d = delta / dist;
                    let st = clip_ray_to_node_shape(c1, s1, d, from_node.node_type);
                    (st, mouse_pos)
                } else {
                    (c1, mouse_pos)
                };

                // Draw glowing preview line
                painter.line_segment(
                    [start, end],
                    Stroke::new(2.5, Color32::from_rgb(45, 212, 191)),
                );

                // Draw targeting node indicator circle at cursor
                painter.circle_filled(end, 5.0, Color32::from_rgb(45, 212, 191));
                painter.circle_stroke(end, 8.0, Stroke::new(1.5, Color32::from_rgb(45, 212, 191)));
            }
        }
    }

    // Handle Deselect on Empty Canvas Background Click
    if response.clicked() && !any_item_clicked {
        state.selected_node_id = None;
        state.selected_line_id = None;
    }

    // Handle Canvas Pan with Left Mouse Drag (on empty background) or Middle Mouse Drag
    if response.dragged_by(PointerButton::Middle)
        || (response.dragged_by(PointerButton::Primary)
            && !any_item_dragged
            && state.tool == CanvasTool::Select
            && state.dragging_node_type.is_none())
    {
        state.pan_offset += response.drag_delta();
    }

    // Drag and Drop Node Payload Handler from Palette to Canvas
    if let Some(node_type) = state.dragging_node_type {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let ghost_size = vec2(100.0 * state.zoom, 40.0 * state.zoom);
            let ghost_rect = Rect::from_center_size(pointer_pos, ghost_size);

            // Draw floating ghost preview box
            painter.rect_filled(
                ghost_rect,
                4.0,
                Color32::from_rgba_unmultiplied(45, 212, 191, 40),
            );
            painter.rect_stroke(
                ghost_rect,
                4.0,
                Stroke::new(2.0, Color32::from_rgb(45, 212, 191)),
            );

            // Drop node on pointer release inside canvas rect
            if ui.input(|i| i.pointer.any_released()) {
                if rect.contains(pointer_pos) {
                    let target_cx = snap_to_grid((pointer_pos.x - rect.min.x - state.pan_offset.x) / state.zoom);
                    let target_cy = snap_to_grid((pointer_pos.y - rect.min.y - state.pan_offset.y) / state.zoom);
                    let canvas_x = target_cx - 50.0;
                    let canvas_y = target_cy - 20.0;
                    let new_id = diagram.add_node(node_type, canvas_x, canvas_y);
                    state.selected_node_id = Some(new_id);
                    state.tool = CanvasTool::Select;
                }
                state.dragging_node_type = None;
            }
        } else if ui.input(|i| i.pointer.any_released()) {
            state.dragging_node_type = None;
        }
    }

    // Floating Zoom HUD Overlay (Bottom-Right corner of canvas)
    let hud_rect = Rect::from_min_size(
        pos2(rect.max.x - 175.0, rect.max.y - 42.0),
        vec2(165.0, 32.0),
    );

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(hud_rect), |ui| {
        egui::Frame::none()
            .fill(if state.is_dark { Color32::from_black_alpha(210) } else { Color32::from_white_alpha(230) })
            .stroke(Stroke::new(1.0, Color32::from_gray(100)))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(6.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("-").strong()).on_hover_text("Uzaklaş (Zoom Out)").clicked() {
                        state.zoom = (state.zoom - 0.1).max(0.25);
                    }

                    if ui.button(RichText::new(format!("{:.0}%", state.zoom * 100.0)).size(11.0))
                        .on_hover_text("Sıfırla (%100 Zoom)")
                        .clicked()
                    {
                        state.zoom = 1.0;
                    }

                    if ui.button(RichText::new("+").strong()).on_hover_text("Yakınlaş (Zoom In)").clicked() {
                        state.zoom = (state.zoom + 0.1).min(3.5);
                    }

                    ui.separator();

                    if ui.button(RichText::new("🎯").size(11.0)).on_hover_text("Diyagramı Merkezle").clicked() {
                        state.needs_centering = true;
                    }
                });
            });
    });

    response
}

fn dist_to_line(point: Option<Pos2>, a: Pos2, b: Pos2) -> f32 {
    let p = match point {
        Some(p) => p,
        None => return f32::MAX,
    };
    let l2 = a.distance_sq(b);
    if l2 == 0.0 {
        return p.distance(a);
    }
    let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    let projection = pos2(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    p.distance(projection)
}

fn render_node_shape(
    painter: &egui::Painter,
    rect: Rect,
    node: &Node,
    is_selected: bool,
    is_active: bool,
    is_dark: bool,
) {
    let fill = if is_dark {
        Color32::from_rgb(35, 42, 54)
    } else {
        Color32::from_rgb(255, 255, 255)
    };

    let border_color = if is_active {
        Color32::from_rgb(45, 212, 191)
    } else if is_selected {
        Color32::from_rgb(251, 191, 36)
    } else {
        match node.node_type {
            NodeType::Start | NodeType::Stop => Color32::from_rgb(52, 211, 153),
            NodeType::Action | NodeType::Definition => Color32::from_rgb(125, 211, 252),
            NodeType::Add | NodeType::Subtract | NodeType::Multiply | NodeType::Divide => {
                Color32::from_rgb(192, 132, 252)
            }
            NodeType::Input => Color32::from_rgb(45, 212, 191),
            NodeType::Output => Color32::from_rgb(251, 146, 60),
            NodeType::IfEqual
            | NodeType::IfGreater
            | NodeType::IfGreaterEqual
            | NodeType::IfLess
            | NodeType::IfLessEqual => Color32::from_rgb(251, 191, 36),
            NodeType::Intersection => Color32::from_rgb(203, 213, 225),
            NodeType::Function => Color32::from_rgb(232, 121, 249),
        }
    };

    let stroke = Stroke::new(if is_active || is_selected { 3.0 } else { 2.0 }, border_color);

    // Glowing highlight box around active executing node
    if is_active {
        let glow_rect = rect.expand(6.0);
        painter.rect_stroke(glow_rect, 6.0, Stroke::new(2.5, Color32::from_rgb(45, 212, 191)));
    }

    match node.node_type {
        NodeType::Start | NodeType::Stop => {
            let rounding = rect.height() / 2.0;
            painter.rect(rect, rounding, fill, stroke);
        }
        NodeType::Action
        | NodeType::Definition
        | NodeType::Add
        | NodeType::Subtract
        | NodeType::Multiply
        | NodeType::Divide
        | NodeType::Function => {
            painter.rect(rect, 4.0, fill, stroke);
            if node.node_type == NodeType::Function {
                // Double vertical bars inside function node
                painter.line_segment(
                    [pos2(rect.min.x + 8.0, rect.min.y), pos2(rect.min.x + 8.0, rect.max.y)],
                    stroke,
                );
                painter.line_segment(
                    [pos2(rect.max.x - 8.0, rect.min.y), pos2(rect.max.x - 8.0, rect.max.y)],
                    stroke,
                );
            }
        }
        NodeType::Input | NodeType::Output => {
            let skew = 12.0;
            let points = vec![
                pos2(rect.min.x + skew, rect.min.y),
                pos2(rect.max.x, rect.min.y),
                pos2(rect.max.x - skew, rect.max.y),
                pos2(rect.min.x, rect.max.y),
            ];
            painter.add(PathShape::convex_polygon(points.clone(), fill, stroke));
        }
        NodeType::IfEqual
        | NodeType::IfGreater
        | NodeType::IfGreaterEqual
        | NodeType::IfLess
        | NodeType::IfLessEqual => {
            let center = rect.center();
            let points = vec![
                pos2(center.x, rect.min.y),
                pos2(rect.max.x, center.y),
                pos2(center.x, rect.max.y),
                pos2(rect.min.x, center.y),
            ];
            painter.add(PathShape::convex_polygon(points, fill, stroke));
        }
        NodeType::Intersection => {
            painter.circle(rect.center(), rect.width() / 2.0, fill, stroke);
            painter.circle_filled(rect.center(), 3.0, border_color);
        }
    }

    // Text content inside shape
    if node.node_type != NodeType::Intersection {
        let text_color = if is_dark { Color32::WHITE } else { Color32::BLACK };

        let primary_text = match node.node_type {
            NodeType::Start => if node.expr1.is_empty() { "başla".to_string() } else { node.expr1.clone() },
            NodeType::Stop => "dur".to_string(),
            NodeType::Input => {
                let e1 = node.expr1.trim();
                let e2 = node.expr2.trim();
                if e1.is_empty() {
                    format!("giriş\n{}", e2)
                } else if e2.is_empty() {
                    format!("giriş\n{}", e1)
                } else {
                    format!("giriş\n{} {}", e1, e2)
                }
            }
            NodeType::Output => {
                let e1 = node.expr1.trim();
                let e2 = node.expr2.trim();
                if e1.is_empty() {
                    format!("çıkış\n{}", e2)
                } else if e2.is_empty() {
                    format!("çıkış\n{}", e1)
                } else {
                    format!("çıkış\n{} {}", e1, e2)
                }
            }
            NodeType::IfEqual => format!("{} = {}", node.expr1, node.expr2),
            NodeType::IfGreater => format!("{} > {}", node.expr1, node.expr2),
            NodeType::IfGreaterEqual => format!("{} >= {}", node.expr1, node.expr2),
            NodeType::IfLess => format!("{} < {}", node.expr1, node.expr2),
            NodeType::IfLessEqual => format!("{} <= {}", node.expr1, node.expr2),
            NodeType::Definition => format!("tanım\n{} = {}", node.expr1, node.expr2),
            NodeType::Add => format!("{} = {} + {}", node.target_var, node.expr1, node.expr2),
            NodeType::Subtract => format!("{} = {} - {}", node.target_var, node.expr1, node.expr2),
            NodeType::Multiply => format!("{} = {} * {}", node.target_var, node.expr1, node.expr2),
            NodeType::Divide => format!("{} = {} / {}", node.target_var, node.expr1, node.expr2),
            NodeType::Action => {
                if !node.target_var.is_empty() {
                    format!("{} = {}", node.target_var, node.expr2)
                } else {
                    format!("{} {}", node.expr1, node.expr2)
                }
            }
            NodeType::Function => format!("func {}\n{}", node.expr1, node.expr2),
            _ => node.label.clone(),
        };

        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            primary_text,
            FontId::proportional(12.0),
            text_color,
        );
    }
}
