//! Arayuz primitifleri: dugmeler, bolum etiketleri, ikon cizimleri, cetvel.

use crate::theme::{Palette, font_display, font_mono};
use egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui,
    Vec2, pos2, vec2,
};

/// Metni tek satirda olcup genislik dondurur.
fn text_width(ui: &Ui, text: &str, font: FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), font, Color32::WHITE)
        .size()
        .x
}

/// Birincil eylem: dolu vermilyon zemin, koyu/acik kontrast metin.
pub fn primary_button(ui: &mut Ui, label: &str, p: &Palette) -> Response {
    let font = font_mono(10.5);
    let width = text_width(ui, label, font.clone()) + 34.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
    let t = ui
        .ctx()
        .animate_bool_with_time(resp.id, resp.hovered(), 0.12);
    let pressed = resp.is_pointer_button_down_on();
    let bg = if pressed {
        p.accent_hover
    } else {
        p.accent.lerp_to_gamma(p.accent_hover, t)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, bg);
    let mut galley = painter.layout_no_wrap(label.to_owned(), font.clone(), p.on_accent);
    if pressed {
        galley = painter.layout_no_wrap(label.to_owned(), font, p.on_accent);
    }
    let offset = if pressed { 1.0 } else { 0.0 };
    painter.galley(
        pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5 + offset,
        ),
        galley,
        p.on_accent,
    );
    resp
}

/// Ikincil eylem: hairline cerceve, seffaf zemin.
pub fn ghost_button(ui: &mut Ui, label: &str, p: &Palette) -> Response {
    let font = font_mono(10.5);
    let width = text_width(ui, label, font.clone()) + 26.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 30.0), Sense::click());
    let t = ui
        .ctx()
        .animate_bool_with_time(resp.id, resp.hovered(), 0.12);
    let painter = ui.painter();
    let border = p.hairline_strong.lerp_to_gamma(p.accent, t);
    painter.rect_filled(rect, 2.0, p.bg.lerp_to_gamma(p.surface, t));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, border), StrokeKind::Inside);
    let color = p.ink.lerp_to_gamma(p.accent, t);
    let galley = painter.layout_no_wrap(label.to_owned(), font, color);
    painter.galley(
        pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        ),
        galley,
        color,
    );
    resp
}

/// Metin bagi: hover'da alt cizgi. Geri donusum gibi yikici eylemlerde kirmizi.
pub fn text_button(ui: &mut Ui, label: &str, p: &Palette, danger: bool) -> Response {
    let font = font_mono(10.5);
    let width = text_width(ui, label, font.clone()) + 4.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
    let t = ui
        .ctx()
        .animate_bool_with_time(resp.id, resp.hovered(), 0.12);
    let base = if danger { p.danger } else { p.ink_muted };
    let hover = if danger { p.accent } else { p.ink };
    let color = base.lerp_to_gamma(hover, t);
    let painter = ui.painter();
    let galley = painter.layout_no_wrap(label.to_owned(), font, color);
    painter.galley(
        pos2(rect.left() + 2.0, rect.center().y - galley.size().y * 0.5),
        galley,
        color,
    );
    if t > 0.01 {
        painter.hline(
            rect.left() + 2.0..=rect.left() + 2.0 + width - 6.0,
            rect.bottom() - 4.0,
            Stroke::new(1.0, color.gamma_multiply(t)),
        );
    }
    resp
}

/// Bolum etiketi: kucuk kare + mono buyuk harf.
pub fn section_label(ui: &mut Ui, p: &Palette, text: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(
        Rect::from_min_size(pos2(rect.left(), rect.center().y - 2.5), Vec2::splat(5.0)),
        0.0,
        p.accent,
    );
    painter.text(
        pos2(rect.left() + 11.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font_mono(9.5),
        p.ink_muted,
    );
}

/// Cerceveli kucuk bilgi parcasi.
pub fn chip(ui: &mut Ui, _p: &Palette, text: &str, color: Color32) {
    let font = font_mono(9.0);
    let width = text_width(ui, text, font.clone()) + 14.0;
    let (rect, _) = ui.allocate_exact_size(vec2(width, 17.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0, color.gamma_multiply(0.55)),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.center().x, rect.center().y),
        Align2::CENTER_CENTER,
        text,
        font,
        color,
    );
}

/// Yatay hairline.
pub fn hairline(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, p.hairline),
    );
}

/// Asagi/yana chevron.
pub fn chevron(painter: &Painter, center: Pos2, size: f32, open: bool, color: Color32) {
    let s = size * 0.5;
    let points = if open {
        vec![
            pos2(center.x - s, center.y - s * 0.45),
            pos2(center.x, center.y + s * 0.55),
            pos2(center.x + s, center.y - s * 0.45),
        ]
    } else {
        vec![
            pos2(center.x - s * 0.45, center.y - s),
            pos2(center.x + s * 0.55, center.y),
            pos2(center.x - s * 0.45, center.y + s),
        ]
    };
    painter.add(Shape::line(points, Stroke::new(1.2, color)));
}

/// Buyutec ikonu.
pub fn magnifier(painter: &Painter, center: Pos2, size: f32, color: Color32) {
    let r = size * 0.32;
    let c = pos2(center.x - size * 0.08, center.y - size * 0.08);
    painter.circle_stroke(c, r, Stroke::new(1.1, color));
    painter.line_segment(
        [
            pos2(c.x + r * 0.7, c.y + r * 0.7),
            pos2(c.x + r * 1.75, c.y + r * 1.75),
        ],
        Stroke::new(1.1, color),
    );
}

/// Klasor silueti (kapali ya da acik).
pub fn folder_glyph(painter: &Painter, rect: Rect, color: Color32) {
    let r = rect.shrink(rect.width() * 0.18);
    let tab_w = r.width() * 0.42;
    painter.add(Shape::line(
        vec![
            pos2(r.left(), r.bottom() - 1.0),
            pos2(r.left(), r.top() + r.height() * 0.22),
            pos2(r.left() + tab_w, r.top() + r.height() * 0.22),
            pos2(r.left() + tab_w + 2.0, r.top()),
            pos2(r.right(), r.top()),
            pos2(r.right(), r.bottom() - 1.0),
            pos2(r.left(), r.bottom() - 1.0),
        ],
        Stroke::new(1.0, color),
    ));
}

/// Kucuk kare nokta.
pub fn warning_glyph(painter: &Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let s = rect.width() * 0.34;
    painter.add(Shape::line(
        vec![
            pos2(c.x, c.y - s),
            pos2(c.x + s, c.y + s * 0.8),
            pos2(c.x - s, c.y + s * 0.8),
            pos2(c.x, c.y - s),
        ],
        Stroke::new(1.1, color),
    ));
    painter.line_segment(
        [pos2(c.x, c.y - s * 0.25), pos2(c.x, c.y + s * 0.25)],
        Stroke::new(1.1, color),
    );
    painter.rect_filled(
        Rect::from_center_size(pos2(c.x, c.y + s * 0.52), Vec2::splat(1.6)),
        0.0,
        color,
    );
}

/// Sol kenardaki derinlik cetveli. `marker` 0..1 arasi bir konumdur.
pub fn ruler(ui: &mut Ui, p: &Palette, marker: Option<f32>) {
    let height = ui.available_height().max(120.0);
    let (rect, _) = ui.allocate_exact_size(vec2(18.0, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, p.panel);
    painter.vline(
        rect.right() - 0.5,
        rect.y_range(),
        Stroke::new(1.0, p.hairline),
    );

    let top = rect.top() + 46.0;
    let mut y = top;
    let mut i = 0;
    while y < rect.bottom() - 8.0 {
        let major = i % 5 == 0;
        let len = if major { 7.0 } else { 3.5 };
        painter.hline(
            rect.right() - 1.0 - len..=rect.right() - 1.0,
            y,
            Stroke::new(1.0, p.hairline_strong),
        );
        if i % 10 == 0 {
            painter.text(
                pos2(rect.left() + 2.0, y - 3.0),
                Align2::LEFT_TOP,
                format!("{i}"),
                font_mono(7.0),
                p.ink_faint,
            );
        }
        y += 8.0;
        i += 1;
    }

    if let Some(t) = marker {
        let y = top + t.clamp(0.0, 1.0) * (rect.bottom() - top - 12.0);
        painter.hline(
            rect.left() + 1.0..=rect.right() - 1.0,
            y,
            Stroke::new(1.0, p.accent),
        );
        painter.rect_filled(
            Rect::from_center_size(pos2(rect.left() + 5.5, y), Vec2::splat(4.0)),
            0.0,
            p.accent,
        );
    }
}

/// Ilerleme cubugu (dolu oran).
pub fn progress_bar(painter: &Painter, rect: Rect, fraction: f32, fill: Color32, track: Color32) {
    painter.rect_filled(rect, 0.0, track);
    let w = rect.width() * fraction.clamp(0.0, 1.0);
    if w > 0.0 {
        painter.rect_filled(
            Rect::from_min_size(rect.min, vec2(w, rect.height())),
            0.0,
            fill,
        );
    }
}

/// Yazi tipi olcegi: buyuk baslik.
pub fn hero_text(ui: &mut Ui, _p: &Palette, text: &str, size: f32, color: Color32) {
    ui.label(
        egui::RichText::new(text)
            .font(font_display(size))
            .color(color),
    );
}
