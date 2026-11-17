//! Calisma alani: agac solda, harita ortada, denetci sagda, durum cubugu altta.

use crate::app::{IskandilApp, SEARCH_ID, UiAction};
use crate::theme::{self, Palette};
use crate::widgets;
use egui::{Align, Align2, Id, Layout, Rect, RichText, ScrollArea, Sense, Stroke, Ui, pos2, vec2};
use iskandil_core::format as fmt;
use iskandil_core::model::SortOrder;
use iskandil_core::{Category, NONE};

const TREE_ROW: f32 = 22.0;
const SEARCH_ROW: f32 = 34.0;

impl IskandilApp {
    pub fn workspace_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let p = self.p;
        self.chrome_ui(ui);
        self.status_bar(ui);

        egui::Panel::left("iskandil-tree")
            .resizable(true)
            .default_size(382.0)
            .size_range(300.0..=560.0)
            .frame(egui::Frame::NONE.fill(p.panel))
            .show(ui, |ui| {
                self.tree_panel(ui, ctx);
            });

        egui::Panel::right("iskandil-inspector")
            .resizable(true)
            .default_size(348.0)
            .size_range(300.0..=460.0)
            .frame(egui::Frame::NONE.fill(p.panel))
            .show(ui, |ui| {
                self.inspector_ui(ui, ctx);
            });

        egui::CentralPanel::no_frame().show(ui, |ui| {
            self.center_ui(ui, ctx);
        });
    }

    // -----------------------------------------------------------------
    // Durum cubugu
    // -----------------------------------------------------------------

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let p = self.p;
        egui::Panel::bottom("iskandil-status")
            .exact_size(30.0)
            .resizable(false)
            .frame(egui::Frame::NONE.fill(p.panel))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter();
                painter.hline(
                    rect.x_range(),
                    rect.top() + 0.5,
                    Stroke::new(1.0, p.hairline),
                );
                let Some(tree) = &self.tree else { return };
                let path = fmt::middle_ellipsis(&tree.root_path, 64);
                painter.text(
                    pos2(rect.left() + 20.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    path,
                    theme::font_mono(8.5),
                    p.ink_muted,
                );
                if tree.partial {
                    painter.text(
                        pos2(rect.left() + 20.0 + 430.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        "KISMİ",
                        theme::font_mono(8.5),
                        p.warning,
                    );
                }
                let mut parts = vec![
                    format!("{} dosya", fmt::count(tree.total_files())),
                    format!("{} klasör", fmt::count(tree.total_dirs())),
                    fmt::bytes(tree.total_bytes()),
                    fmt::duration(tree.elapsed),
                    format!(
                        "ortalama {}",
                        fmt::rate_count(tree.total_files(), tree.elapsed)
                    ),
                ];
                if tree.errors > 0 {
                    parts.push(format!("{} okunamadı", fmt::count(tree.errors)));
                }
                let text = parts.join("   ·   ");
                painter.text(
                    pos2(rect.right() - 20.0, rect.center().y),
                    Align2::RIGHT_CENTER,
                    text,
                    theme::font_mono(8.5),
                    p.ink_faint,
                );
            });
    }

    // -----------------------------------------------------------------
    // Agac paneli
    // -----------------------------------------------------------------

    fn tree_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let p = self.p;
        self.ensure_rows();
        self.ensure_search();

        ui.add_space(12.0);
        let Some(tree) = &self.tree else { return };
        let (header, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(
            Rect::from_min_size(
                pos2(header.left() + 16.0, header.center().y - 2.5),
                vec2(5.0, 5.0),
            ),
            0.0,
            p.accent,
        );
        painter.text(
            pos2(header.left() + 27.0, header.center().y),
            Align2::LEFT_CENTER,
            "DOSYA AĞACI",
            theme::font_mono(9.5),
            p.ink_muted,
        );
        painter.text(
            pos2(header.right() - 16.0, header.center().y),
            Align2::RIGHT_CENTER,
            fmt::count(tree.total_files()),
            theme::font_mono(8.5),
            p.ink_faint,
        );

        ui.add_space(6.0);
        search_field(ui, &p, &mut self.search);
        self.ensure_search();
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                RichText::new("SIRALA")
                    .font(theme::font_mono(9.0))
                    .color(p.ink_muted),
            );
            let previous = self.sort_order;
            egui::ComboBox::from_id_salt("tree-sort")
                .selected_text(match self.sort_order {
                    SortOrder::SizeDescending => "Boyut: büyükten küçüğe",
                    SortOrder::SizeAscending => "Boyut: küçükten büyüğe",
                    SortOrder::NameAscending => "Ad: A–Z",
                    SortOrder::NameDescending => "Ad: Z–A",
                    SortOrder::ModifiedDescending => "Tarih: yeniden eskiye",
                    SortOrder::ModifiedAscending => "Tarih: eskiden yeniye",
                })
                .show_ui(ui, |ui| {
                    for (order, label) in [
                        (SortOrder::SizeDescending, "Boyut: büyükten küçüğe"),
                        (SortOrder::SizeAscending, "Boyut: küçükten büyüğe"),
                        (SortOrder::NameAscending, "Ad: A–Z"),
                        (SortOrder::NameDescending, "Ad: Z–A"),
                        (SortOrder::ModifiedDescending, "Tarih: yeniden eskiye"),
                        (SortOrder::ModifiedAscending, "Tarih: eskiden yeniye"),
                    ] {
                        ui.selectable_value(&mut self.sort_order, order, label);
                    }
                });
            if previous != self.sort_order {
                self.rows_cache = None;
                self.search_cache = None;
            }
        });
        self.ensure_rows();
        self.ensure_search();
        ui.add_space(8.0);
        widgets::hairline(ui, &p);

        let searching = !self.search.trim().is_empty();
        let mut actions: Vec<UiAction> = Vec::new();

        if searching {
            let hits: &[u32] = self
                .search_cache
                .as_ref()
                .map(|c| c.hits.as_slice())
                .unwrap_or(&[]);
            let (meta, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
            ui.painter().text(
                pos2(meta.left() + 16.0, meta.center().y),
                Align2::LEFT_CENTER,
                if hits.is_empty() {
                    "EŞLEŞME YOK".to_string()
                } else {
                    format!("{} SONUÇ", fmt::count(hits.len() as u64))
                },
                theme::font_mono(8.5),
                p.ink_faint,
            );

            let tree = self.tree.as_ref().unwrap();
            let selected = self.selected;
            ui.spacing_mut().item_spacing.y = 0.0;
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, SEARCH_ROW, hits.len(), |ui, range| {
                    let width = ui.available_width();
                    for idx in range {
                        let id = hits[idx];
                        let node = tree.node(id);
                        let (rect, resp) =
                            ui.allocate_exact_size(vec2(width, SEARCH_ROW), Sense::click());
                        let hovered = resp.hovered();
                        let selected_row = selected == Some(id);
                        paint_row_bg(ui, &p, rect, hovered, selected_row);
                        let icon_x = rect.left() + 26.0;
                        paint_node_mark(ui, &p, rect, icon_x, node.is_dir, node.category);
                        let name_x = icon_x + 14.0;
                        let size_text = fmt::bytes(node.size);
                        let size_galley = ui.painter().layout_no_wrap(
                            size_text,
                            theme::font_mono(9.0),
                            p.ink_muted,
                        );
                        let size_w = size_galley.size().x;
                        let clip = Rect::from_min_max(
                            pos2(name_x, rect.top() + 3.0),
                            pos2(rect.right() - size_w - 22.0, rect.bottom()),
                        );
                        let cp = ui.painter().with_clip_rect(clip);
                        cp.text(
                            pos2(name_x, rect.top() + 4.0),
                            Align2::LEFT_TOP,
                            node.name.as_ref(),
                            theme::font_body(12.0),
                            p.ink,
                        );
                        let parent_path = if node.parent != NONE {
                            tree.path_of(node.parent)
                        } else {
                            tree.root_path.clone()
                        };
                        cp.text(
                            pos2(name_x, rect.top() + 18.0),
                            Align2::LEFT_TOP,
                            fmt::middle_ellipsis(&parent_path, 90),
                            theme::font_mono(8.0),
                            p.ink_faint,
                        );
                        ui.painter().galley(
                            pos2(rect.right() - 16.0 - size_w, rect.center().y - 5.0),
                            size_galley,
                            p.ink_muted,
                        );
                        if resp.clicked() {
                            if node.is_dir {
                                actions.push(UiAction::View(id));
                            } else {
                                actions.push(UiAction::View(node.parent));
                                actions.push(UiAction::Select(id));
                            }
                        }
                        resp.context_menu(|ui| {
                            node_menu(ui, &p, tree, id, &mut actions);
                        });
                    }
                });
        } else {
            let rows = self
                .rows_cache
                .as_ref()
                .map(|c| c.rows.as_slice())
                .unwrap_or(&[]);
            let tree = self.tree.as_ref().unwrap();
            let selected = self.selected;
            let expanded = &self.expanded;
            ui.spacing_mut().item_spacing.y = 0.0;
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, TREE_ROW, rows.len(), |ui, range| {
                    let width = ui.available_width();
                    for idx in range {
                        let (id, depth) = rows[idx];
                        let node = tree.node(id);
                        let (rect, resp) =
                            ui.allocate_exact_size(vec2(width, TREE_ROW), Sense::click());
                        let hovered = resp.hovered();
                        let selected_row = selected == Some(id);
                        let hover_t = ui.ctx().animate_bool_with_time(
                            Id::unique(("tree-row", id)),
                            hovered,
                            0.07,
                        );
                        if selected_row {
                            ui.painter().rect_filled(rect, 0.0, p.selection);
                            ui.painter().rect_filled(
                                Rect::from_min_size(rect.min, vec2(2.0, rect.height())),
                                0.0,
                                p.accent,
                            );
                        } else if hover_t > 0.01 {
                            ui.painter()
                                .rect_filled(rect, 0.0, p.bg.gamma_multiply(hover_t));
                        }

                        let indent = 9.0 + depth as f32 * 13.0;
                        let has_children = node.is_dir && node.first_child != NONE;
                        if has_children {
                            let open = expanded.contains(&id);
                            widgets::chevron(
                                ui.painter(),
                                pos2(rect.left() + indent + 5.0, rect.center().y),
                                7.0,
                                open,
                                if selected_row { p.ink } else { p.ink_faint },
                            );
                        }
                        let icon_x = rect.left() + indent + 20.0;
                        paint_node_mark(ui, &p, rect, icon_x, node.is_dir, node.category);
                        let name_x = rect.left() + indent + 32.0;
                        let size_text = fmt::bytes(node.size);
                        let size_galley = ui.painter().layout_no_wrap(
                            size_text,
                            theme::font_mono(9.0),
                            p.ink_muted,
                        );
                        let size_w = size_galley.size().x;
                        let name_clip = Rect::from_min_max(
                            pos2(name_x, rect.top()),
                            pos2(rect.right() - size_w - 18.0, rect.bottom()),
                        );
                        ui.painter().with_clip_rect(name_clip).text(
                            pos2(name_x, rect.center().y),
                            Align2::LEFT_CENTER,
                            node.name.as_ref(),
                            theme::font_body(12.0),
                            p.ink,
                        );
                        ui.painter().galley(
                            pos2(rect.right() - 10.0 - size_w, rect.center().y - 5.0),
                            size_galley,
                            if selected_row { p.ink } else { p.ink_muted },
                        );

                        // Alt mini pay cubugu
                        let share = tree.share_of_parent(id);
                        if share > 0.0 {
                            let bar_w = (rect.width() - indent - 40.0).max(0.0) * share;
                            let color = if node.is_dir {
                                p.hairline_strong
                            } else {
                                theme::category_color(node.category, p.mode)
                            };
                            let bar = Rect::from_min_size(
                                pos2(rect.left() + indent + 32.0, rect.bottom() - 2.0),
                                vec2(bar_w.min(rect.width() * 0.6), 1.0),
                            );
                            ui.painter().rect_filled(
                                bar,
                                0.0,
                                color.gamma_multiply(if selected_row { 0.85 } else { 0.5 }),
                            );
                        }

                        if resp.clicked() {
                            let pointer_x = resp
                                .interact_pointer_pos()
                                .map(|pos| pos.x)
                                .unwrap_or(rect.right());
                            if has_children && pointer_x < rect.left() + indent + 13.0 {
                                actions.push(UiAction::Toggle(id));
                            } else if node.is_dir {
                                actions.push(UiAction::View(id));
                            } else {
                                actions.push(UiAction::Select(id));
                            }
                        }
                        if resp.double_clicked() && node.is_dir {
                            actions.push(UiAction::Toggle(id));
                        }
                        resp.context_menu(|ui| {
                            node_menu(ui, &p, tree, id, &mut actions);
                        });
                    }
                });
        }

        for action in actions {
            self.perform(action, ctx);
        }
    }

    // -----------------------------------------------------------------
    // Denetci
    // -----------------------------------------------------------------

    fn inspector_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let p = self.p;
        self.ensure_stats();
        let Some(tree) = &self.tree else { return };
        let focus_id = self.selected.unwrap_or(self.view_root);
        let node = tree.node(focus_id);
        let stats_types = self
            .folder_stats
            .as_ref()
            .map(|s| s.types.clone())
            .unwrap_or_default();
        let top = self
            .folder_stats
            .as_ref()
            .map(|s| s.top.clone())
            .unwrap_or_default();
        let mut delete_id: Option<u32> = None;
        let mut actions: Vec<UiAction> = Vec::new();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    panel_heading(ui, &p, "SEÇİM", "");
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width() - 16.0, 20.0),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_width(ui.available_width());
                            ui.add(
                                egui::Label::new(
                                    RichText::new(node.name.as_ref())
                                        .font(theme::font_display(20.0))
                                        .color(p.ink),
                                )
                                .truncate(),
                            );
                        },
                    );
                });
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let path = tree.path_of(focus_id);
                    ui.add(
                        egui::Label::new(
                            RichText::new(fmt::middle_ellipsis(&path, 70))
                                .font(theme::font_mono(8.5))
                                .color(p.ink_faint),
                        )
                        .truncate(),
                    );
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let (rect, _) = ui.allocate_exact_size(
                        vec2(ui.available_width() - 16.0, 38.0),
                        Sense::hover(),
                    );
                    ui.painter().text(
                        pos2(rect.left(), rect.top() + 2.0),
                        Align2::LEFT_TOP,
                        fmt::bytes(node.size),
                        theme::font_display(26.0),
                        p.ink,
                    );
                    let parent_bytes = if node.parent != NONE {
                        tree.node(node.parent).size.max(1)
                    } else {
                        tree.total_bytes().max(1)
                    };
                    ui.painter().text(
                        pos2(rect.right(), rect.top() + 12.0),
                        Align2::RIGHT_TOP,
                        if node.parent == NONE {
                            match tree.volume_total {
                                Some(total) => {
                                    format!("birimin {}", fmt::percent(node.size, total))
                                }
                                None => format!("kökün {}", fmt::percent(node.size, parent_bytes)),
                            }
                        } else {
                            format!("ebeveynin {}", fmt::percent(node.size, parent_bytes))
                        },
                        theme::font_mono(8.5),
                        p.ink_faint,
                    );
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let color = if node.is_dir {
                        p.ink_muted
                    } else {
                        theme::category_color(node.category, p.mode)
                    };
                    let label = if node.is_dir {
                        "KLASÖR".to_string()
                    } else {
                        node.category.label_tr().to_uppercase()
                    };
                    widgets::chip(ui, &p, &label, color);
                    if node.reparse {
                        ui.add_space(4.0);
                        widgets::chip(ui, &p, "BAĞLANTI", p.warning);
                    }
                });

                ui.add_space(14.0);
                meta_row(ui, &p, "DOSYA", &fmt::count(node.files));
                meta_row(ui, &p, "KLASÖR", &fmt::count(node.dirs));
                meta_row(ui, &p, "DEĞİŞTİRİLDİ", &fmt::timestamp(node.mtime));

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    if widgets::ghost_button(ui, "GEZGİNDE AÇ", &p).clicked() {
                        actions.push(UiAction::OpenExplorer(focus_id));
                    }
                    ui.add_space(4.0);
                    if widgets::text_button(ui, "YOLU KOPYALA", &p, false).clicked() {
                        actions.push(UiAction::CopyPath(focus_id));
                    }
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    if focus_id != tree.root
                        && widgets::text_button(ui, "GERİ DÖNÜŞÜME TAŞI", &p, true).clicked()
                    {
                        delete_id = Some(focus_id);
                    }
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    widgets::hairline(ui, &p);
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    panel_heading(ui, &p, "EN BÜYÜK DOSYALAR", "");
                });
                ui.add_space(2.0);
                if top.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            RichText::new("Dosya bulunamadı.")
                                .font(theme::font_body(11.5))
                                .color(p.ink_faint),
                        );
                    });
                }
                for (i, id) in top.iter().take(5).enumerate() {
                    let node = tree.node(*id);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        let (rect, resp) = ui.allocate_exact_size(
                            vec2(ui.available_width() - 16.0, 18.0),
                            Sense::click(),
                        );
                        let hover = resp.hovered();
                        if hover {
                            ui.painter().rect_filled(rect, 0.0, p.bg);
                        }
                        ui.painter().text(
                            pos2(rect.left(), rect.center().y),
                            Align2::LEFT_CENTER,
                            format!("{:02}", i + 1),
                            theme::font_mono(7.5),
                            p.ink_faint,
                        );
                        let size_galley = ui.painter().layout_no_wrap(
                            fmt::bytes(node.size),
                            theme::font_mono(8.5),
                            p.ink_muted,
                        );
                        let size_w = size_galley.size().x;
                        let clip = Rect::from_min_max(
                            pos2(rect.left() + 22.0, rect.top()),
                            pos2(rect.right() - size_w - 8.0, rect.bottom()),
                        );
                        ui.painter().with_clip_rect(clip).text(
                            pos2(rect.left() + 22.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            node.name.as_ref(),
                            theme::font_body(11.0),
                            if hover { p.ink } else { p.ink_muted },
                        );
                        ui.painter().galley(
                            pos2(rect.right() - size_w, rect.center().y - 4.5),
                            size_galley,
                            p.ink_muted,
                        );
                        if resp.clicked() {
                            actions.push(UiAction::View(node.parent));
                            actions.push(UiAction::Select(*id));
                        }
                    });
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    widgets::hairline(ui, &p);
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    panel_heading(
                        ui,
                        &p,
                        "TÜRLER",
                        &format!("{} ÖĞE", fmt::count(stats_types.len() as u64)),
                    );
                });
                ui.add_space(4.0);
                let view_bytes = tree.node(self.view_root).size.max(1);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let (bar, _) = ui.allocate_exact_size(
                        vec2(ui.available_width() - 16.0, 6.0),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(bar, 0.0, p.hairline);
                    let mut x = bar.left();
                    for (cat, bytes, _) in stats_types.iter().take(8) {
                        let w = bar.width() * (*bytes as f32 / view_bytes as f32);
                        if w <= 0.0 {
                            continue;
                        }
                        let seg = Rect::from_min_size(pos2(x, bar.top()), vec2(w, bar.height()));
                        ui.painter()
                            .rect_filled(seg, 0.0, theme::category_color(*cat, p.mode));
                        x += w;
                    }
                });
                ui.add_space(4.0);
                if stats_types.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            RichText::new("Bu klasörde dosya yok.")
                                .font(theme::font_body(11.5))
                                .color(p.ink_faint),
                        );
                    });
                } else {
                    let summary = stats_types
                        .iter()
                        .take(4)
                        .map(|(cat, bytes, _)| {
                            format!("{} {}", cat.label_tr(), fmt::percent(*bytes, view_bytes))
                        })
                        .collect::<Vec<_>>()
                        .join("  ·  ");
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        let (rect, _) = ui.allocate_exact_size(
                            vec2(ui.available_width() - 16.0, 15.0),
                            Sense::hover(),
                        );
                        ui.painter().text(
                            pos2(rect.left(), rect.center().y),
                            Align2::LEFT_CENTER,
                            summary,
                            theme::font_mono(8.5),
                            p.ink_muted,
                        );
                    });
                }
                ui.add_space(10.0);
            });

        for action in actions {
            self.perform(action, ctx);
        }
        if let Some(id) = delete_id {
            self.perform(UiAction::AskDelete(id), ctx);
        }
    }

    // -----------------------------------------------------------------
    // Orta alan
    // -----------------------------------------------------------------

    fn center_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let p = self.p;
        self.ensure_stats();
        let Some(tree) = &self.tree else { return };
        let view = self.view_root;
        let node = tree.node(view);
        let mut actions: Vec<UiAction> = Vec::new();

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            let mut chain = vec![view];
            let mut cur = view;
            while cur != tree.root {
                cur = tree.parent_of(cur);
                if cur == NONE {
                    break;
                }
                chain.push(cur);
            }
            chain.reverse();
            for (i, id) in chain.iter().enumerate() {
                if i > 0 {
                    ui.label(
                        RichText::new("/")
                            .font(theme::font_mono(9.0))
                            .color(p.ink_faint),
                    );
                }
                let is_last = i + 1 == chain.len();
                let label = if *id == tree.root {
                    tree.root_name.clone()
                } else {
                    tree.node(*id).name.to_string()
                };
                if is_last {
                    ui.label(
                        RichText::new(label)
                            .font(theme::font_mono(9.0))
                            .color(p.ink),
                    );
                } else {
                    let resp = widgets::text_button(ui, &label, &p, false);
                    if resp.clicked() {
                        actions.push(UiAction::View(*id));
                    }
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(18.0);
                if view != tree.root {
                    let resp = widgets::text_button(ui, "YUKARI ↑", &p, false);
                    if resp.clicked() {
                        if let Some(parent) = self.parent_id(view) {
                            actions.push(UiAction::View(parent));
                        }
                    }
                }
            });
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            let avail = ui.available_width() - 18.0;
            ui.allocate_ui_with_layout(
                vec2(avail * 0.62, 58.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_max_width(ui.available_width());
                    ui.add(
                        egui::Label::new(
                            RichText::new(node.name.as_ref())
                                .font(theme::font_display(27.0))
                                .color(p.ink),
                        )
                        .truncate(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(tree.path_of(view))
                                .font(theme::font_mono(8.5))
                                .color(p.ink_faint),
                        )
                        .truncate(),
                    );
                },
            );
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(fmt::bytes(node.size))
                                .font(theme::font_mono(21.0))
                                .color(p.ink),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let mut detail = if node.is_dir {
                            format!(
                                "{} dosya · {} klasör",
                                fmt::count(node.files),
                                fmt::count(node.dirs)
                            )
                        } else {
                            "dosya".to_string()
                        };
                        if view == tree.root {
                            if let Some(total) = tree.volume_total {
                                detail.push_str(&format!(
                                    " · {} dolu",
                                    fmt::percent(node.size, total)
                                ));
                            }
                        }
                        ui.label(
                            RichText::new(detail)
                                .font(theme::font_mono(8.5))
                                .color(p.ink_faint),
                        );
                    });
                });
            });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            widgets::hairline(ui, &p);
        });
        ui.add_space(6.0);

        let area = Rect::from_min_max(
            ui.cursor().min + vec2(18.0, 0.0),
            pos2(ui.max_rect().right() - 18.0, ui.max_rect().bottom() - 6.0),
        );
        if area.width() > 40.0 && area.height() > 60.0 {
            let mut map_rect = area;
            map_rect.max.y -= 26.0;
            self.paint_treemap(ui, map_rect, ctx, &mut actions);
            paint_legend(ui, &p, area, self);
        }

        for action in actions {
            self.perform(action, ctx);
        }
    }
}

// ---------------------------------------------------------------------
// Kucuk parcalar
// ---------------------------------------------------------------------

fn panel_heading(ui: &mut Ui, p: &Palette, left: &str, right: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
    ui.painter().rect_filled(
        Rect::from_min_size(pos2(rect.left(), rect.center().y - 2.5), vec2(5.0, 5.0)),
        0.0,
        p.accent,
    );
    ui.painter().text(
        pos2(rect.left() + 11.0, rect.center().y),
        Align2::LEFT_CENTER,
        left,
        theme::font_mono(9.5),
        p.ink_muted,
    );
    if !right.is_empty() {
        ui.painter().text(
            pos2(rect.right(), rect.center().y),
            Align2::RIGHT_CENTER,
            right,
            theme::font_mono(8.5),
            p.ink_faint,
        );
    }
}

fn meta_row(ui: &mut Ui, p: &Palette, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let (rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width() - 16.0, 19.0), Sense::hover());
        ui.painter().text(
            pos2(rect.left(), rect.center().y),
            Align2::LEFT_CENTER,
            label,
            theme::font_mono(8.5),
            p.ink_faint,
        );
        ui.painter().text(
            pos2(rect.right(), rect.center().y),
            Align2::RIGHT_CENTER,
            value,
            theme::font_mono(9.0),
            p.ink_muted,
        );
    });
}

fn paint_row_bg(ui: &Ui, p: &Palette, rect: Rect, hovered: bool, selected: bool) {
    if selected {
        ui.painter().rect_filled(rect, 0.0, p.selection);
        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, vec2(2.0, rect.height())),
            0.0,
            p.accent,
        );
    } else if hovered {
        ui.painter().rect_filled(rect, 0.0, p.bg);
    }
}

fn paint_node_mark(ui: &Ui, p: &Palette, rect: Rect, x: f32, is_dir: bool, category: Category) {
    if is_dir {
        widgets::folder_glyph(
            ui.painter(),
            Rect::from_center_size(pos2(x, rect.center().y), vec2(12.0, 11.0)),
            p.ink_faint,
        );
    } else {
        ui.painter().rect_filled(
            Rect::from_center_size(pos2(x, rect.center().y), vec2(6.0, 6.0)),
            0.0,
            theme::category_color(category, p.mode),
        );
    }
}

fn search_field(ui: &mut Ui, p: &Palette, search: &mut String) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        let outer_w = ui.available_width() - 14.0;
        egui::Frame::NONE
            .fill(p.bg)
            .stroke(Stroke::new(1.0, p.hairline))
            .corner_radius(egui::CornerRadius::same(2))
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                ui.set_width(outer_w - 18.0);
                ui.horizontal(|ui| {
                    let (icon, _) = ui.allocate_exact_size(vec2(13.0, 13.0), Sense::hover());
                    widgets::magnifier(ui.painter(), icon.center(), 12.0, p.ink_faint);
                    let w = (ui.available_width() - 10.0).max(60.0);
                    ui.add_sized(
                        [w, 18.0],
                        egui::TextEdit::singleline(search)
                            .id(Id::unique(SEARCH_ID))
                            .font(theme::font_mono(9.5))
                            .text_color(p.ink)
                            .hint_text(
                                RichText::new("ada göre ara")
                                    .font(theme::font_mono(9.0))
                                    .color(p.ink_faint),
                            )
                            .frame(egui::Frame::NONE)
                            .margin(egui::Margin::ZERO)
                            .desired_width(f32::INFINITY),
                    );
                });
            });
    });
}

fn node_menu(
    ui: &mut Ui,
    p: &Palette,
    tree: &iskandil_core::Tree,
    id: u32,
    actions: &mut Vec<UiAction>,
) {
    let node = tree.node(id);
    if ui.button("Gezginde aç").clicked() {
        actions.push(UiAction::OpenExplorer(id));
        ui.close();
    }
    if ui.button("Yolu kopyala").clicked() {
        actions.push(UiAction::CopyPath(id));
        ui.close();
    }
    if node.is_dir {
        ui.separator();
        if ui.button("Haritada göster").clicked() {
            actions.push(UiAction::View(id));
            ui.close();
        }
    }
    if id != tree.root {
        ui.separator();
        if ui
            .button(RichText::new("Geri dönüşüme taşı").color(p.danger))
            .clicked()
        {
            actions.push(UiAction::AskDelete(id));
            ui.close();
        }
    }
}

fn paint_legend(ui: &mut Ui, p: &Palette, area: Rect, app: &IskandilApp) {
    let legend_rect = Rect::from_min_size(
        pos2(area.left(), area.bottom() - 24.0),
        vec2(area.width(), 24.0),
    );
    ui.painter().hline(
        legend_rect.x_range(),
        legend_rect.top() + 0.5,
        Stroke::new(1.0, p.hairline),
    );
    let mut x = legend_rect.left();
    let types = app
        .folder_stats
        .as_ref()
        .map(|s| s.types.as_slice())
        .unwrap_or(&[]);
    for (cat, bytes, _) in types.iter().take(6) {
        if x + 150.0 > legend_rect.right() {
            break;
        }
        ui.painter().rect_filled(
            Rect::from_min_size(pos2(x, legend_rect.center().y - 3.0), vec2(6.0, 6.0)),
            0.0,
            theme::category_color(*cat, p.mode),
        );
        ui.painter().text(
            pos2(x + 12.0, legend_rect.center().y),
            Align2::LEFT_CENTER,
            format!("{}  {}", cat.label_tr(), fmt::bytes(*bytes)),
            theme::font_mono(8.5),
            p.ink_muted,
        );
        x += 12.0
            + ui.painter()
                .layout_no_wrap(
                    format!("{}  {}", cat.label_tr(), fmt::bytes(*bytes)),
                    theme::font_mono(8.5),
                    p.ink_muted,
                )
                .size()
                .x
            + 18.0;
    }
    ui.painter().text(
        pos2(legend_rect.right(), legend_rect.center().y),
        Align2::RIGHT_CENTER,
        "RENK = DOSYA TÜRÜ",
        theme::font_mono(7.5),
        p.ink_faint,
    );
}
