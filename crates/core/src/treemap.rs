//! Saf squarified treemap yerlesimi.
//!
//! `build`, secili klasorun cocuklarini alanla orantili dikdortgenlere boler;
//! bir klasor yeterince buyukse icine girip alt cocuklarini da yerleştirir.
//! Cizim ve renk kararlari arayuz katmanina aittir; bu modul yalnizca geometri uretir.

use crate::model::{NONE, Tree};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl TRect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(self) -> f32 {
        self.y + self.h
    }

    pub fn area(self) -> f32 {
        self.w.max(0.0) * self.h.max(0.0)
    }

    pub fn is_empty(self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    pub fn inset(self, d: f32) -> Self {
        Self {
            x: self.x + d,
            y: self.y + d,
            w: (self.w - d * 2.0).max(0.0),
            h: (self.h - d * 2.0).max(0.0),
        }
    }

    pub fn contains(self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    pub fn overlap_area(self, other: TRect) -> f32 {
        let w = (self.right().min(other.right()) - self.x.max(other.x)).max(0.0);
        let h = (self.bottom().min(other.bottom()) - self.y.max(other.y)).max(0.0);
        w * h
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub id: u32,
    pub parent: u32,
    pub depth: u8,
    pub rect: TRect,
    pub size: u64,
    pub is_dir: bool,
    /// Klasorun icine girilip cocuklari yerlestirildi mi?
    pub subdivided: bool,
    /// Ustte klasor basligi icin serit ayrildi mi?
    pub header: bool,
    /// `id == NONE` ise bu, sigmayan ogelerin toplu kutusudur.
    pub remainder: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct TreemapOptions {
    pub max_depth: u8,
    pub min_leaf_area: f32,
    pub max_tiles: usize,
    pub max_children: usize,
    pub gap: f32,
}

impl Default for TreemapOptions {
    fn default() -> Self {
        Self {
            max_depth: 6,
            min_leaf_area: 700.0,
            max_tiles: 4000,
            max_children: 240,
            gap: 1.5,
        }
    }
}

/// Klasor basligi seridinin yuksekligi (piksel).
pub const HEADER_H: f32 = 15.0;

/// Secili klasorun agacindan treemap dossegleri uretir.
pub fn build(tree: &Tree, root: u32, rect: TRect, opts: TreemapOptions) -> Vec<Tile> {
    let mut out = Vec::new();
    if rect.w < 2.0 || rect.h < 2.0 {
        return out;
    }
    layout_dir(tree, root, rect, 0, opts, &mut out);
    out
}

fn layout_dir(
    tree: &Tree,
    dir: u32,
    rect: TRect,
    depth: u8,
    opts: TreemapOptions,
    out: &mut Vec<Tile>,
) {
    if out.len() >= opts.max_tiles || rect.w < 3.0 || rect.h < 3.0 {
        return;
    }
    let node = tree.node(dir);
    if node.first_child == NONE {
        return;
    }
    let mut kids = tree.children_sorted(dir);
    if kids.is_empty() {
        return;
    }

    let mut total: u64 = 0;
    for &id in &kids {
        total = total.saturating_add(tree.node(id).size);
    }
    if total == 0 {
        return;
    }

    let mut remainder_count = 0u32;
    let mut remainder_size = 0u64;
    if kids.len() > opts.max_children {
        let extra = kids.split_off(opts.max_children);
        remainder_count = extra.len() as u32;
        for id in extra {
            remainder_size = remainder_size.saturating_add(tree.node(id).size);
        }
    }

    let mut ids: Vec<u32> = kids.clone();
    if remainder_count > 0 {
        ids.push(NONE);
    }

    let mut areas: Vec<f32> = Vec::with_capacity(ids.len());
    for &id in &ids {
        let size = if id == NONE {
            remainder_size
        } else {
            tree.node(id).size
        };
        areas.push(size as f32);
    }

    let rects = squarify(&areas, rect, opts.gap);
    for (idx, &id) in ids.iter().enumerate() {
        let r = rects[idx];
        if r.is_empty() {
            continue;
        }
        let is_rem = id == NONE;
        let size = if is_rem {
            remainder_size
        } else {
            tree.node(id).size
        };
        let is_dir = !is_rem && tree.node(id).is_dir;
        out.push(Tile {
            id,
            parent: dir,
            depth,
            rect: r,
            size,
            is_dir,
            subdivided: false,
            header: false,
            remainder: if is_rem { remainder_count } else { 0 },
        });
        let tile_index = out.len() - 1;
        if is_dir
            && depth + 1 < opts.max_depth
            && out.len() < opts.max_tiles
            && r.w > 26.0
            && r.h > 26.0
            && r.area() >= opts.min_leaf_area
        {
            let header = (depth == 0 && r.w >= 110.0 && r.h >= 66.0)
                || (depth == 1 && r.w >= 200.0 && r.h >= 120.0);
            let inner = if header {
                out[tile_index].header = true;
                TRect::new(
                    r.x + opts.gap,
                    r.y + HEADER_H + opts.gap,
                    (r.w - opts.gap * 2.0).max(0.0),
                    (r.h - HEADER_H - opts.gap * 2.0).max(0.0),
                )
            } else {
                r.inset(opts.gap)
            };
            layout_dir(tree, id, inner, depth + 1, opts, out);
            out[tile_index].subdivided = out.len() > tile_index + 1;
        }
    }
}

/// Alanlari (piksel kare) verilen dikdortgene squarified olarak yerlestirir.
/// Sonuc, girdi sirasini korur.
pub fn squarify(areas: &[f32], rect: TRect, gap: f32) -> Vec<TRect> {
    let mut out = vec![TRect::default(); areas.len()];
    if rect.is_empty() {
        return out;
    }
    let total: f32 = areas.iter().filter(|a| **a > 0.0).sum();
    if total <= 0.0 || !total.is_finite() {
        return out;
    }

    let scale = rect.area() / total;
    if !scale.is_finite() || scale <= 0.0 {
        return out;
    }
    let scaled: Vec<f32> = areas.iter().map(|a| (a * scale).max(0.0)).collect();

    let mut order: Vec<usize> = (0..areas.len()).filter(|&i| scaled[i] > 0.0).collect();
    order.sort_by(|&a, &b| {
        scaled[b]
            .partial_cmp(&scaled[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });

    let mut free = rect;
    let mut i = 0usize;
    while i < order.len() {
        if free.is_empty() {
            break;
        }
        let short = free.w.min(free.h).max(0.05);
        let mut row: Vec<usize> = Vec::new();
        let mut row_area = 0.0f32;
        let mut worst = f32::INFINITY;
        while i < order.len() {
            let idx = order[i];
            let a = scaled[idx];
            if a <= 0.0 {
                i += 1;
                continue;
            }
            let candidate_area = row_area + a;
            let mut trial = row.clone();
            trial.push(idx);
            let ratio = worst_ratio(&trial, &scaled, short);
            if !row.is_empty() && ratio > worst * 1.0001 {
                break;
            }
            row = trial;
            row_area = candidate_area;
            worst = ratio;
            i += 1;
        }
        if row.is_empty() {
            break;
        }

        let half_gap = gap * 0.5;
        if free.w <= free.h {
            // Yatay serit: genislik boyunca, kalinlik = alan / genislik.
            let thickness = (row_area / free.w.max(0.05)).min(free.h);
            let mut x = free.x;
            for (n, &idx) in row.iter().enumerate() {
                let w = if n + 1 == row.len() {
                    free.right() - x
                } else {
                    scaled[idx] / thickness.max(0.05)
                };
                let w = w.clamp(0.0, (free.right() - x).max(0.0));
                out[idx] = TRect::new(
                    x + half_gap,
                    free.y + half_gap,
                    (w - gap).max(0.0),
                    (thickness - gap).max(0.0),
                );
                x += w;
            }
            free.y += thickness;
            free.h = (free.h - thickness).max(0.0);
        } else {
            // Dikey sutun: yukseklik boyunca, kalinlik = alan / yukseklik.
            let thickness = (row_area / free.h.max(0.05)).min(free.w);
            let mut y = free.y;
            for (n, &idx) in row.iter().enumerate() {
                let h = if n + 1 == row.len() {
                    free.bottom() - y
                } else {
                    scaled[idx] / thickness.max(0.05)
                };
                let h = h.clamp(0.0, (free.bottom() - y).max(0.0));
                out[idx] = TRect::new(
                    free.x + half_gap,
                    y + half_gap,
                    (thickness - gap).max(0.0),
                    (h - gap).max(0.0),
                );
                y += h;
            }
            free.x += thickness;
            free.w = (free.w - thickness).max(0.0);
        }
    }
    out
}

fn worst_ratio(row: &[usize], scaled: &[f32], short: f32) -> f32 {
    if row.is_empty() {
        return f32::INFINITY;
    }
    let mut max = f32::MIN;
    let mut min = f32::MAX;
    let mut sum = 0.0;
    for &idx in row {
        let a = scaled[idx];
        max = max.max(a);
        min = min.min(a);
        sum += a;
    }
    if sum <= 0.0 || min <= 0.0 {
        return f32::INFINITY;
    }
    let s2 = short * short;
    (s2 * max / (sum * sum)).max(sum * sum / (s2 * min))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_tiles_are_proportional() {
        let rect = TRect::new(0.0, 0.0, 400.0, 300.0);
        let tiles = squarify(&[75.0, 25.0], rect, 0.0);
        assert!((tiles[0].area() - 90_000.0).abs() < 1.0);
        assert!((tiles[1].area() - 30_000.0).abs() < 1.0);
        for t in &tiles {
            assert!(t.x >= rect.x - 0.01 && t.y >= rect.y - 0.01);
            assert!(t.right() <= rect.right() + 0.01);
            assert!(t.bottom() <= rect.bottom() + 0.01);
        }
    }

    #[test]
    fn tiles_do_not_overlap() {
        let rect = TRect::new(10.0, 20.0, 640.0, 480.0);
        let areas = [500.0, 300.0, 120.0, 80.0, 40.0, 20.0, 10.0, 5.0];
        let tiles = squarify(&areas, rect, 1.0);
        for i in 0..tiles.len() {
            for j in i + 1..tiles.len() {
                assert!(
                    tiles[i].overlap_area(tiles[j]) < 0.01,
                    "{i} ve {j} cakisiyor"
                );
            }
        }
        let sum: f32 = tiles.iter().map(|t| t.area()).sum();
        assert!(sum <= rect.area() + 1.0);
        assert!(sum > rect.area() * 0.75);
    }

    #[test]
    fn gap_creates_gutters() {
        let rect = TRect::new(0.0, 0.0, 100.0, 100.0);
        let tiles = squarify(&[50.0, 50.0], rect, 2.0);
        // Iki tile arasinda en az bir bosluk olmali.
        let a = tiles[0];
        let b = tiles[1];
        let overlap = a.overlap_area(b);
        assert_eq!(overlap, 0.0);
    }

    #[test]
    fn zero_total_is_empty() {
        let tiles = squarify(&[0.0, 0.0], TRect::new(0.0, 0.0, 100.0, 100.0), 1.0);
        assert!(tiles.iter().all(|t| t.is_empty()));
    }

    #[test]
    fn single_tile_fills_rect_with_gap() {
        let rect = TRect::new(0.0, 0.0, 200.0, 120.0);
        let tiles = squarify(&[1.0], rect, 2.0);
        assert!(tiles[0].w >= 197.0 && tiles[0].h >= 117.0);
    }
}
