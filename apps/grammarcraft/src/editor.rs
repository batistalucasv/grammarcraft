use egui::text::CCursor;
use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2};
use grammarcraft_core::Issue;

use crate::state::{GrammarCraftApp, PaneFocusTarget};
use crate::theme_bridge::{UnderlinePattern, category_style, with_alpha};

pub fn show(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    let avail = ui.available_size();
    let editor_height = (avail.y - 8.0).max(200.0);
    show_with_rows(app, ui, 18, editor_height);
}

pub(crate) fn show_with_rows(
    app: &mut GrammarCraftApp,
    ui: &mut egui::Ui,
    rows: usize,
    max_height: f32,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.strong("Document");
            let stats = app.session.doc().stats();
            ui.label(format!(
                "• {} words • {} sentences • {} issue(s)",
                stats.words,
                stats.sentences,
                app.issues().len()
            ));
        });
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .id_salt("editor_scroll")
            .max_height(max_height)
            .show(ui, |ui| {
                let output = egui::TextEdit::multiline(&mut app.editor_text)
                    .id_source("document_editor")
                    .desired_width(f32::INFINITY)
                    .desired_rows(rows)
                    .hint_text("Type or paste your text here for live proofing...")
                    .show(ui);

                ui.ctx().accesskit_node_builder(output.response.id, |node| {
                    node.set_label("Document Editor");
                });

                if app.focus_request == Some(PaneFocusTarget::Editor) {
                    output.response.request_focus();
                    app.focus_request = None;
                }

                if output.response.changed() {
                    app.set_text_and_check();
                }

                paint_issue_overlays(app, ui, &output);
                handle_editor_pointer(app, ui, &output);
            });
    });
}

fn paint_issue_overlays(
    app: &GrammarCraftApp,
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
) {
    let painter = ui.painter_at(output.text_clip_rect);
    let text = &app.editor_text;

    if let Some((hl_start, hl_end)) = app.highlight_span {
        for rect in span_rects(&output.galley, output.galley_pos, text, hl_start, hl_end) {
            painter.rect_filled(rect.expand(1.5), 2.0, with_alpha(app.colors.accent, 44));
        }
    }

    for issue in app.issues() {
        let style = category_style(issue.category, issue.severity, &app.colors);
        let start = issue.span.start();
        let end = issue.span.end();
        let is_selected = app.selected_issue_id == Some(issue.id.get());

        for rect in span_rects(&output.galley, output.galley_pos, text, start, end) {
            if is_selected {
                painter.rect_filled(rect.expand(1.0), 2.0, with_alpha(style.color, 36));
            }
            paint_redundant_underline(&painter, rect, style.color, style.pattern);
        }
    }
}

fn handle_editor_pointer(
    app: &mut GrammarCraftApp,
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
) {
    let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    if !output.text_clip_rect.contains(pointer) {
        return;
    }
    let hovered = hit_test_issue(app, &output.galley, output.galley_pos, pointer);
    if let Some((issue, style_label, badge)) = hovered {
        let issue_id = issue.id.get();
        let span = (issue.span.start(), issue.span.end());
        let tooltip_text = format!(
            "{badge} {style_label}: {}\n{}",
            issue.title, issue.explanation
        );
        if ui.input(|i| i.pointer.primary_clicked()) {
            app.select_issue(issue_id, span);
        } else {
            <egui::Response as Clone>::clone(&output.response).on_hover_text(tooltip_text);
        }
    }
}

fn hit_test_issue<'a>(
    app: &'a GrammarCraftApp,
    galley: &egui::Galley,
    origin: Pos2,
    pointer: Pos2,
) -> Option<(&'a Issue, &'static str, &'static str)> {
    let text = &app.editor_text;
    for issue in app.issues() {
        let rects = span_rects(galley, origin, text, issue.span.start(), issue.span.end());
        if rects.iter().any(|r| r.expand(3.0).contains(pointer)) {
            let st = category_style(issue.category, issue.severity, &app.colors);
            return Some((issue, st.category_label, st.severity_badge));
        }
    }
    None
}

fn span_rects(
    galley: &egui::Galley,
    origin: Pos2,
    text: &str,
    byte_start: usize,
    byte_end: usize,
) -> Vec<Rect> {
    if byte_start >= byte_end || byte_end > text.len() {
        return Vec::new();
    }
    if !text.is_char_boundary(byte_start) || !text.is_char_boundary(byte_end) {
        return Vec::new();
    }
    let Some(prefix) = text.get(..byte_start) else {
        return Vec::new();
    };
    let Some(span_text) = text.get(byte_start..byte_end) else {
        return Vec::new();
    };
    let char_start = prefix.chars().count();
    let char_end = char_start + span_text.chars().count();

    let start_pos = galley.pos_from_cursor(CCursor::new(char_start));
    let end_pos = galley.pos_from_cursor(CCursor::new(char_end));

    let min_x = origin.x + start_pos.min.x;
    let max_x = origin.x + end_pos.min.x.max(start_pos.min.x + 6.0);
    let min_y = origin.y + start_pos.min.y;
    let max_y = origin.y + start_pos.max.y.max(start_pos.min.y + 14.0);

    if (start_pos.min.y - end_pos.min.y).abs() < 2.0 {
        vec![Rect::from_min_max(
            Pos2::new(min_x, min_y),
            Pos2::new(max_x, max_y),
        )]
    } else {
        vec![Rect::from_min_max(
            Pos2::new(min_x, min_y),
            Pos2::new(origin.x + galley.rect.width().max(min_x + 12.0), max_y),
        )]
    }
}

fn paint_redundant_underline(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    pattern: UnderlinePattern,
) {
    let y = rect.bottom() - 1.0;
    let x0 = rect.left();
    let x1 = rect.right();
    if x1 <= x0 {
        return;
    }

    match pattern {
        UnderlinePattern::Wavy => {
            let mut pts = Vec::new();
            let step = 3.0;
            let mut x = x0;
            let mut up = true;
            while x < x1 {
                let offset = if up { -1.2 } else { 1.2 };
                pts.push(Pos2::new(x, y + offset));
                x += step;
                up = !up;
            }
            pts.push(Pos2::new(x1, y));
            painter.add(Shape::line(pts, Stroke::new(1.3, color)));
        }
        UnderlinePattern::Double => {
            painter.line_segment(
                [Pos2::new(x0, y - 1.5), Pos2::new(x1, y - 1.5)],
                Stroke::new(1.0, color),
            );
            painter.line_segment(
                [Pos2::new(x0, y + 1.0), Pos2::new(x1, y + 1.0)],
                Stroke::new(1.0, color),
            );
        }
        UnderlinePattern::Dashed => {
            let mut x = x0;
            while x < x1 {
                let end = (x + 5.0).min(x1);
                painter.line_segment(
                    [Pos2::new(x, y), Pos2::new(end, y)],
                    Stroke::new(1.5, color),
                );
                x += 8.0;
            }
        }
        UnderlinePattern::Dotted => {
            let mut x = x0 + 1.0;
            while x <= x1 {
                painter.circle_filled(Pos2::new(x, y), 1.1, color);
                x += 4.0;
            }
        }
        UnderlinePattern::SolidBracket => {
            painter.line_segment(
                [Pos2::new(x0, y), Pos2::new(x1, y)],
                Stroke::new(1.6, color),
            );
            painter.line_segment(
                [Pos2::new(x0, y - 2.5), Pos2::new(x0, y)],
                Stroke::new(1.4, color),
            );
            painter.line_segment(
                [Pos2::new(x1, y - 2.5), Pos2::new(x1, y)],
                Stroke::new(1.4, color),
            );
        }
    }
    let _ = Vec2::ZERO;
}
