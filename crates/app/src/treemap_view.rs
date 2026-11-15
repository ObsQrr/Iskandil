//! Treemap cizimi, hover, secim ve baglam menusu.

use crate::app::{IskandilApp, UiAction};
use crate::theme::{self, Palette};
use egui::{Align2, Color32, Id, Order, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use iskandil_core::NONE;
use iskandil_core::format as fmt;
use iskandil_core::treemap::Tile;

impl IskandilApp {
    pub fn paint_treemap(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        ctx: &egui::Context,
        actions: &mut Vec<UiAction>,
    ) {
        let p = self.p;
        self.ensure_tiles(rect);
        let Some(tree) = &self.tree else { return };
        let tiles: &[Tile] = self
            .tiles_cache
            .as_ref()
            .map(|c| c.tiles.as_slice())
            .unwrap_or(&[]);
        let view = self.view_root;
        let selected = self.selected;
        let resp = ui.interact(rect, ui.scope_id().with("treemap"), Sense::click());
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, 0.0, p.bg);
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, p.hairline), StrokeKind::Inside);

        if tiles.is_empty() {
            let c = rect.center();
            painter.hline(
                rect.left() + rect.width() * 0.35..=rect.right() - rect.width() * 0.35,
                c.y - 18.0,
                Stroke::new(1.0, p.hairline),
            );
            painter.text(
                pos2(c.x, c.y + 2.0),
                Align2::CENTER_CENTER,
                "Bu klasörde haritalanacak veri yok.",
                theme::font_body(12.0),
                p.ink_muted,
            );
            painter.text(
                pos2(c.x, c.y + 20.0),
                Align2::CENTER_CENTER,
                "boş ya da yalnızca sıfır baytlık öğeler var",
                theme::font_mono(8.5),
                p.ink_faint,
            );
            return;
        }

        let hover_pos = resp.hover_pos();
        let hovered = hover_pos.and_then(|pos| {
            let local = pos - rect.min;
            tiles
                .iter()
                .rposition(|t| t.rect.contains(local.x, local.y))
        });

        // 1) Zeminler
        for t in tiles {
            let r = map_rect(rect, t.rect);
            if r.width() < 1.0 || r.height() < 1.0 {
                continue;
            }
            if t.id == NONE {
                painter.rect_filled(r, 0.0, p.surface_hi);
                hatch(&painter, r, p.hairline_strong);
                continue;
            }
            if t.is_dir {
                painter.rect_filled(r, 0.0, p.folder);
                painter.rect_stroke(
                    r,
                    0.0,
                    Stroke::new(1.0, p.hairline_strong),
                    StrokeKind::Inside,
                );
            } else {
                let color = theme::category_color(tree.node(t.id).category, p.mode);
                painter.rect_filled(r, 0.0, color);
            }
        }

        // 2) Yaprak etiketleri (dosyalar ve bolunmemis klasorler)
        for t in tiles {
            let r = map_rect(rect, t.rect);
            if r.width() < 12.0 || r.height() < 8.0 {
                continue;
            }
            if t.id == NONE {
                if r.width() > 56.0 && r.height() > 22.0 {
                    painter.text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        format!("{} öğe", fmt::count(t.remainder as u64)),
                        theme::font_mono(8.5),
                        p.ink_muted,
                    );
                }
                continue;
            }
            if t.subdivided {
                continue;
            }
            let node = tree.node(t.id);
            if r.width() > 44.0 && r.height() > 15.0 {
                let inset = r.shrink(4.0);
                let cp = painter.with_clip_rect(inset);
                cp.text(
                    pos2(inset.left(), inset.top()),
                    Align2::LEFT_TOP,
                    node.name.as_ref(),
                    theme::font_body(9.5),
                    if t.is_dir { p.ink } else { p.tile_text },
                );
                if r.width() > 78.0 && r.height() > 31.0 {
                    cp.text(
                        pos2(inset.left(), inset.top() + 14.0),
                        Align2::LEFT_TOP,
                        fmt::bytes(t.size),
                        theme::font_mono(8.0),
                        if t.is_dir {
                            p.ink_muted
                        } else {
                            p.tile_text_dim
                        },
                    );
                }
            }
        }

        // 2b) Klasor baslik seritleri (en ustte, cocuklarin uzerini kapatmaz)
        for t in tiles {
            if !t.subdivided {
                continue;
            }
            let r = map_rect(rect, t.rect);
            let node = tree.node(t.id);
            if t.header {
                folder_header(&painter, &p, r, &node.name, t.size);
            } else if t.depth == 0 && r.width() > 64.0 && r.height() > 26.0 {
                folder_compact_tag(&painter, &p, r, &node.name);
            }
        }

        // 3) Hover ve secim
        if let Some(idx) = hovered {
            let t = &tiles[idx];
            let r = map_rect(rect, t.rect);
            let overlay = if p.mode == iskandil_core::settings::ThemeMode::Light {
                Color32::from_black_alpha(12)
            } else {
                Color32::from_white_alpha(14)
            };
            painter.rect_filled(r, 0.0, overlay);
        }
        if let Some(sel) = selected {
            if sel != view {
                if let Some(t) = tiles.iter().find(|t| t.id == sel) {
                    let r = map_rect(rect, t.rect);
                    painter.rect_stroke(r, 0.0, Stroke::new(2.0, p.accent), StrokeKind::Inside);
                }
            }
        }
        if let Some(idx) = hovered {
            let t = &tiles[idx];
            let r = map_rect(rect, t.rect);
            let stroke = if selected == Some(t.id) {
                Stroke::new(2.0, p.accent)
            } else {
                Stroke::new(1.0, p.ink)
            };
            painter.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
        }

        // 4) Etkilesim
        if let Some(pos) = hover_pos {
            let local = pos - rect.min;
            if tiles.iter().any(|t| t.rect.contains(local.x, local.y)) {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
        }

        let menu_key = Id::unique("iskandil-treemap-menu-target");
        if resp.secondary_clicked() {
            let target = hovered.map(|i| tiles[i].id).unwrap_or(NONE);
            ui.data_mut(|d| d.insert_temp(menu_key, target));
            if target != NONE {
                actions.push(UiAction::Select(target));
            }
        }

        if resp.clicked() {
            match hovered {
                Some(i) => {
                    let t = &tiles[i];
                    if t.id != NONE {
                        actions.push(UiAction::Select(t.id));
                        if resp.double_clicked() && t.is_dir {
                            actions.push(UiAction::View(t.id));
                        }
                    }
                }
                None => actions.push(UiAction::Select(view)),
            }
        }

        resp.context_menu(|ui| {
            let target = ui.data(|d| d.get_temp::<u32>(menu_key)).unwrap_or(NONE);
            if target == NONE {
                ui.label(
                    egui::RichText::new("Boş alan")
                        .font(theme::font_mono(9.0))
                        .color(p.ink_faint),
                );
                return;
            }
            let node = tree.node(target);
            ui.label(
                egui::RichText::new(format!(
                    "{} · {}",
                    node.name.as_ref(),
                    fmt::bytes(node.size)
                ))
                .font(theme::font_mono(9.0))
                .color(p.ink_muted),
            );
            ui.separator();
            if ui.button("Gezginde aç").clicked() {
                actions.push(UiAction::OpenExplorer(target));
                ui.close();
            }
            if ui.button("Yolu kopyala").clicked() {
                actions.push(UiAction::CopyPath(target));
                ui.close();
            }
            if node.is_dir && ui.button("İçine gir").clicked() {
                actions.push(UiAction::View(target));
                ui.close();
            }
            if target != tree.root {
                ui.separator();
                if ui
                    .button(egui::RichText::new("Geri dönüşüme taşı").color(p.danger))
                    .clicked()
                {
                    actions.push(UiAction::AskDelete(target));
                    ui.close();
                }
            }
        });

        // 5) Ipucu karti
        if let (Some(pos), Some(idx)) = (hover_pos, hovered) {
            let t = &tiles[idx];
            let screen = ctx.content_rect();
            let tip_pos = pos2(
                (pos.x + 16.0)
                    .min(screen.right() - 300.0)
                    .max(screen.left() + 8.0),
                (pos.y + 18.0)
                    .min(screen.bottom() - 130.0)
                    .max(screen.top() + 8.0),
            );
            let view_bytes = tree.node(view).size.max(1);
            let name = if t.id == NONE {
                format!("{} öğe", fmt::count(t.remainder as u64))
            } else {
                tree.node(t.id).name.to_string()
            };
            let detail = if t.id == NONE {
                "haritaya sığmayan öğeler".to_string()
            } else {
                let node = tree.node(t.id);
                if node.is_dir {
                    format!(
                        "{} dosya · {} klasör",
                        fmt::count(node.files),
                        fmt::count(node.dirs)
                    )
                } else {
                    node.category.label_tr().to_string()
                }
            };
            egui::Area::new(Id::unique("iskandil-tile-tip"))
                .order(Order::Tooltip)
                .interactable(false)
                .fixed_pos(tip_pos)
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(p.panel)
                        .stroke(Stroke::new(1.0, p.hairline_strong))
                        .inner_margin(egui::Margin::same(10))
                        .corner_radius(egui::CornerRadius::same(2))
                        .show(ui, |ui| {
                            ui.set_max_width(280.0);
                            ui.label(
                                egui::RichText::new(name)
                                    .font(theme::font_display(13.0))
                                    .color(p.ink),
                            );
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(fmt::bytes(t.size))
                                        .font(theme::font_mono(11.0))
                                        .color(p.ink),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "· {}",
                                        fmt::percent(t.size, view_bytes)
                                    ))
                                    .font(theme::font_mono(9.0))
                                    .color(p.ink_muted),
                                );
                            });
                            ui.label(
                                egui::RichText::new(detail)
                                    .font(theme::font_body(10.5))
                                    .color(p.ink_muted),
                            );
                            if t.is_dir && t.id != NONE {
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new("çift tık: içine gir")
                                        .font(theme::font_mono(8.0))
                                        .color(p.ink_faint),
                                );
                            }
                        });
                });
        }
    }
}

fn map_rect(origin: Rect, r: iskandil_core::treemap::TRect) -> Rect {
    Rect::from_min_size(
        pos2(origin.left() + r.x, origin.top() + r.y),
        vec2(r.w, r.h),
    )
}

fn hatch(painter: &egui::Painter, rect: Rect, color: Color32) {
    let painter = painter.with_clip_rect(rect);
    let mut x = rect.left() - rect.height();
    while x < rect.right() {
        painter.line_segment(
            [pos2(x, rect.bottom()), pos2(x + rect.height(), rect.top())],
            Stroke::new(1.0, color.gamma_multiply(0.5)),
        );
        x += 7.0;
    }
}

fn folder_compact_tag(painter: &egui::Painter, p: &Palette, rect: Rect, name: &str) {
    let galley = painter.layout_no_wrap(name.to_owned(), theme::font_mono(8.0), p.ink);
    let w = (galley.size().x + 12.0).min(rect.width() - 6.0);
    let tag = Rect::from_min_size(pos2(rect.left() + 3.0, rect.top() + 3.0), vec2(w, 14.0));
    painter.rect_filled(tag, 0.0, p.bg);
    painter.rect_stroke(tag, 0.0, Stroke::new(1.0, p.hairline), StrokeKind::Inside);
    painter.rect_filled(
        Rect::from_min_size(tag.min, vec2(2.0, tag.height())),
        0.0,
        p.accent,
    );
    let cp = painter.with_clip_rect(tag.shrink2(vec2(5.0, 1.0)));
    cp.text(
        pos2(tag.left() + 6.0, tag.center().y),
        Align2::LEFT_CENTER,
        name,
        theme::font_mono(8.0),
        p.ink,
    );
}

fn folder_header(painter: &egui::Painter, p: &Palette, rect: Rect, name: &str, size: u64) {
    let strip = Rect::from_min_size(
        rect.min,
        vec2(rect.width(), iskandil_core::treemap::HEADER_H),
    );
    painter.rect_filled(strip, 0.0, p.bg);
    painter.hline(
        strip.x_range(),
        strip.bottom() - 0.5,
        Stroke::new(1.0, p.hairline),
    );
    painter.rect_filled(
        Rect::from_min_size(strip.min, vec2(2.0, strip.height())),
        0.0,
        p.accent,
    );
    let label = format!("{}  {}", name, fmt::bytes(size));
    let cp = painter.with_clip_rect(strip.shrink2(vec2(7.0, 1.0)));
    cp.text(
        pos2(strip.left() + 7.0, strip.center().y),
        Align2::LEFT_CENTER,
        label,
        theme::font_mono(8.5),
        p.ink,
    );
}
