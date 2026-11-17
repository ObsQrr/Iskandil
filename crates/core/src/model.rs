//! Dosya agaci: arena tabanli dugumler, bagli liste cocuklar, toplamlar.
//!
//! Dugumler tarama sirasinda eklenir; bir cocugun kimligi her zaman ebeveyninden
//! buyuktur. Bu sayede tum agac tersten gezilerek boyut toplamlari tek geciste
//! hesaplanabilir.

use crate::categories::Category;
use crate::format::{contains_folded, fold_tr};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const NONE: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    SizeDescending,
    SizeAscending,
    NameAscending,
    NameDescending,
    ModifiedDescending,
    ModifiedAscending,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: Box<str>,
    pub parent: u32,
    /// Ilk cocugun kimligi.
    pub first_child: u32,
    /// Son cocugun kimligi (ekleme O(1)).
    pub last_child: u32,
    /// Ayni ebeveyndeki sonraki kardes.
    pub next_sibling: u32,
    /// Dosya icin bayt; klasor icin alt agac toplami.
    pub size: u64,
    /// Alt agactaki dosya sayisi (dosyanin kendisi 1).
    pub files: u64,
    /// Alt agactaki klasor sayisi (kendisi haric).
    pub dirs: u64,
    pub mtime: i64,
    pub category: Category,
    pub is_dir: bool,
    /// NTFS yeniden ayristirma noktasi (juncture/symlink) - icine girilmedi.
    pub reparse: bool,
}

impl Node {
    pub fn new_file(
        name: impl Into<Box<str>>,
        parent: u32,
        size: u64,
        mtime: i64,
        category: Category,
        reparse: bool,
    ) -> Self {
        Self {
            name: name.into(),
            parent,
            first_child: NONE,
            last_child: NONE,
            next_sibling: NONE,
            size,
            files: 1,
            dirs: 0,
            mtime,
            category,
            is_dir: false,
            reparse,
        }
    }

    pub fn new_dir(name: impl Into<Box<str>>, parent: u32, mtime: i64, reparse: bool) -> Self {
        Self {
            name: name.into(),
            parent,
            first_child: NONE,
            last_child: NONE,
            next_sibling: NONE,
            size: 0,
            files: 0,
            dirs: 0,
            mtime,
            category: Category::Other,
            is_dir: true,
            reparse,
        }
    }
}

/// Bir agacin tamami.
#[derive(Clone, Debug)]
pub struct Tree {
    pub nodes: Vec<Node>,
    pub root: u32,
    /// Gosterim yolu, ornegin `C:\Users\Kayon`.
    pub root_path: String,
    /// Kok klasorun gorunen adi (`C:\` icin `C:\`).
    pub root_name: String,
    pub elapsed: std::time::Duration,
    pub errors: u64,
    pub threads: usize,
    /// Iptal edilerek bitirildi.
    pub partial: bool,
    /// Surucu kapasitesi (kok bir surucu ise).
    pub volume_total: Option<u64>,
}

impl Tree {
    pub fn new(
        root_path: String,
        root_name: String,
        root_mtime: i64,
        volume_total: Option<u64>,
        threads: usize,
    ) -> Self {
        let root_node = Node::new_dir(root_name.clone(), NONE, root_mtime, false);
        Self {
            nodes: vec![root_node],
            root: 0,
            root_path,
            root_name,
            elapsed: std::time::Duration::ZERO,
            errors: 0,
            threads,
            partial: false,
            volume_total,
        }
    }

    #[inline]
    pub fn node(&self, id: u32) -> &Node {
        &self.nodes[id as usize]
    }

    #[inline]
    pub fn node_mut(&mut self, id: u32) -> &mut Node {
        &mut self.nodes[id as usize]
    }

    pub fn total_bytes(&self) -> u64 {
        self.nodes[self.root as usize].size
    }

    pub fn total_files(&self) -> u64 {
        self.nodes[self.root as usize].files
    }

    pub fn total_dirs(&self) -> u64 {
        self.nodes[self.root as usize].dirs
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }

    /// Bagli listeyi dolasarak cocuklari dondurur.
    pub fn children(&self, id: u32) -> ChildIter<'_> {
        ChildIter {
            tree: self,
            next: self.nodes[id as usize].first_child,
        }
    }

    pub fn child_count(&self, id: u32) -> usize {
        let mut n = 0;
        let mut cur = self.nodes[id as usize].first_child;
        while cur != NONE {
            n += 1;
            cur = self.nodes[cur as usize].next_sibling;
        }
        n
    }

    /// Cocuklari boyuta gore azalan, esitlikte ada gore sirali dondurur.
    pub fn children_sorted(&self, id: u32) -> Vec<u32> {
        self.children_sorted_by(id, SortOrder::SizeDescending)
    }

    pub fn children_sorted_by(&self, id: u32, order: SortOrder) -> Vec<u32> {
        let mut out: Vec<u32> = self.children(id).collect();
        self.sort_ids(&mut out, order);
        out
    }

    pub fn sort_ids(&self, ids: &mut [u32], order: SortOrder) {
        ids.sort_unstable_by(|&a, &b| {
            let na = &self.nodes[a as usize];
            let nb = &self.nodes[b as usize];
            let name = || na.name.to_lowercase().cmp(&nb.name.to_lowercase());
            let primary = match order {
                SortOrder::SizeDescending => nb.size.cmp(&na.size),
                SortOrder::SizeAscending => na.size.cmp(&nb.size),
                SortOrder::NameAscending => name(),
                SortOrder::NameDescending => name().reverse(),
                SortOrder::ModifiedDescending => nb.mtime.cmp(&na.mtime),
                SortOrder::ModifiedAscending => na.mtime.cmp(&nb.mtime),
            };
            primary.then_with(name).then_with(|| a.cmp(&b))
        });
    }

    pub fn is_ancestor(&self, ancestor: u32, node: u32) -> bool {
        if ancestor == node {
            return true;
        }
        let mut cur = node;
        while cur != NONE {
            if cur == ancestor {
                return true;
            }
            cur = self.nodes[cur as usize].parent;
        }
        false
    }

    pub fn depth_of(&self, id: u32) -> usize {
        let mut depth = 0;
        let mut cur = id;
        while cur != NONE && cur != self.root {
            depth += 1;
            cur = self.nodes[cur as usize].parent;
        }
        depth
    }

    /// Tam gosterim yolu. Kok icin `root_path`.
    pub fn path_of(&self, id: u32) -> String {
        if id == self.root {
            return self.root_path.clone();
        }
        let mut parts: Vec<&str> = Vec::new();
        let mut cur = id;
        while cur != NONE && cur != self.root {
            parts.push(&self.nodes[cur as usize].name);
            cur = self.nodes[cur as usize].parent;
        }
        parts.reverse();
        let root = self.root_path.trim_end_matches('\\');
        if parts.is_empty() {
            root.to_string()
        } else {
            format!("{root}\\{}", parts.join("\\"))
        }
    }

    /// Ust klasorun kimligi (kokte NONE).
    pub fn parent_of(&self, id: u32) -> u32 {
        self.nodes[id as usize].parent
    }

    /// Kokten baslayarak dosyalari dolasir, en buyuk `limit` tanesini dondurur.
    pub fn top_files(&self, root: u32, limit: usize) -> Vec<u32> {
        let mut heap: BinaryHeap<Reverse<(u64, u32)>> = BinaryHeap::with_capacity(limit + 1);
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let node = &self.nodes[id as usize];
            if !node.is_dir {
                heap.push(Reverse((node.size, id)));
                if heap.len() > limit {
                    heap.pop();
                }
                continue;
            }
            let mut cur = node.first_child;
            while cur != NONE {
                stack.push(cur);
                cur = self.nodes[cur as usize].next_sibling;
            }
        }
        let mut out: Vec<u32> = heap.into_iter().map(|Reverse((_, id))| id).collect();
        out.sort_unstable_by(|&a, &b| {
            self.nodes[b as usize]
                .size
                .cmp(&self.nodes[a as usize].size)
        });
        out
    }

    /// Alt agactaki dosya turlerinin bayt ve adet dagilimi.
    pub fn type_breakdown(&self, root: u32) -> Vec<(Category, u64, u64)> {
        let mut bytes = [0u64; Category::COUNT];
        let mut files = [0u64; Category::COUNT];
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let node = &self.nodes[id as usize];
            if node.is_dir {
                let mut cur = node.first_child;
                while cur != NONE {
                    stack.push(cur);
                    cur = self.nodes[cur as usize].next_sibling;
                }
            } else {
                let idx = node.category.index();
                bytes[idx] += node.size;
                files[idx] += 1;
            }
        }
        let mut out: Vec<(Category, u64, u64)> = Category::ALL
            .into_iter()
            .map(|c| (c, bytes[c.index()], files[c.index()]))
            .collect();
        out.retain(|(_, b, f)| *b > 0 || *f > 0);
        out.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
        out
    }

    /// Ada gore arama; sonuclari boyuta gore azalan dondurur.
    pub fn find(&self, query: &str, limit: usize) -> Vec<u32> {
        self.find_sorted(query, limit, SortOrder::SizeDescending)
    }

    pub fn find_sorted(&self, query: &str, limit: usize, order: SortOrder) -> Vec<u32> {
        let needle = fold_tr(query);
        if needle.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<u32> = Vec::new();
        let mut stack = vec![self.root];
        while let Some(id) = stack.pop() {
            let node = &self.nodes[id as usize];
            // Kok adi tam yolu tekrar etmesin.
            if id != self.root && contains_folded(&node.name, &needle) {
                hits.push(id);
            }
            if node.is_dir {
                let mut cur = node.first_child;
                while cur != NONE {
                    stack.push(cur);
                    cur = self.nodes[cur as usize].next_sibling;
                }
            }
        }
        self.sort_ids(&mut hits, order);
        hits.truncate(limit);
        hits
    }

    /// Dugumu agactan cikarir ve ust atalarinin toplamlarini duzeltir.
    /// Alt agac dugumleri arenada kalir ama artik erisilemez.
    pub fn remove(&mut self, id: u32) -> Option<u64> {
        if id == self.root || id as usize >= self.nodes.len() {
            return None;
        }
        let (parent, size, files, dirs, is_dir) = {
            let node = &self.nodes[id as usize];
            (node.parent, node.size, node.files, node.dirs, node.is_dir)
        };
        if parent == NONE {
            return None;
        }

        // Bagli listeden cikar.
        let mut prev = NONE;
        let mut cur = self.nodes[parent as usize].first_child;
        while cur != NONE && cur != id {
            prev = cur;
            cur = self.nodes[cur as usize].next_sibling;
        }
        if cur != id {
            return None;
        }
        let next = self.nodes[id as usize].next_sibling;
        if prev == NONE {
            self.nodes[parent as usize].first_child = next;
        } else {
            self.nodes[prev as usize].next_sibling = next;
        }
        if self.nodes[parent as usize].last_child == id {
            self.nodes[parent as usize].last_child = prev;
        }

        // Ust atalarin toplamlarini dus.
        let mut anc = parent;
        while anc != NONE {
            let n = &mut self.nodes[anc as usize];
            n.size = n.size.saturating_sub(size);
            n.files = n.files.saturating_sub(files);
            n.dirs = n.dirs.saturating_sub(if is_dir { 1 + dirs } else { 0 });
            anc = n.parent;
        }
        Some(size)
    }

    /// Ayni ebeveyn icindeki pay (0..1).
    pub fn share_of_parent(&self, id: u32) -> f32 {
        let parent = self.nodes[id as usize].parent;
        if parent == NONE {
            return 1.0;
        }
        let total = self.nodes[parent as usize].size;
        if total == 0 {
            0.0
        } else {
            self.nodes[id as usize].size as f32 / total as f32
        }
    }

    /// Testler icin tutarlilik kontrolu (kokten erisilebilen dugumler).
    #[cfg(test)]
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("agac bos".into());
        }
        if self.nodes[self.root as usize].parent != NONE {
            return Err("kok ebeveyni NONE olmali".into());
        }
        let mut seen = vec![false; self.nodes.len()];
        let mut stack = vec![self.root];
        while let Some(id) = stack.pop() {
            if seen[id as usize] {
                return Err(format!("dugum {id}: birden fazla kez goruldu"));
            }
            seen[id as usize] = true;
            let mut cur = self.nodes[id as usize].first_child;
            while cur != NONE {
                if self.nodes[cur as usize].parent != id {
                    return Err(format!("dugum {cur}: ebeveyn baglantisi yanlis"));
                }
                stack.push(cur);
                cur = self.nodes[cur as usize].next_sibling;
            }
        }
        for (i, node) in self.nodes.iter().enumerate() {
            if !seen[i] || i == self.root as usize {
                continue;
            }
            let parent = node.parent as usize;
            if self.nodes[parent].size < node.size {
                return Err(format!("dugum {i}: ebeveyn boyutu kucuk"));
            }
        }
        Ok(())
    }
}

pub struct ChildIter<'a> {
    tree: &'a Tree,
    next: u32,
}

impl Iterator for ChildIter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.next == NONE {
            return None;
        }
        let id = self.next;
        self.next = self.tree.nodes[id as usize].next_sibling;
        Some(id)
    }
}

/// Tarama sonrasi toplamlari tek geciste hesaplar (tersten dolasim).
pub fn aggregate(nodes: &mut [Node]) {
    for i in (1..nodes.len()).rev() {
        let (parent, size, files, dirs, is_dir) = {
            let n = &nodes[i];
            (n.parent, n.size, n.files, n.dirs, n.is_dir)
        };
        if parent == NONE {
            continue;
        }
        let p = &mut nodes[parent as usize];
        p.size += size;
        p.files += files;
        p.dirs += dirs + u64::from(is_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_child(tree: &mut Tree, parent: u32, node: Node) -> u32 {
        let id = tree.nodes.len() as u32;
        let prev_last = tree.nodes[parent as usize].last_child;
        let mut node = node;
        node.parent = parent;
        tree.nodes.push(node);
        if prev_last != NONE {
            tree.nodes[prev_last as usize].next_sibling = id;
        }
        let p = &mut tree.nodes[parent as usize];
        if p.first_child == NONE {
            p.first_child = id;
        }
        p.last_child = id;
        id
    }

    fn sample_tree() -> Tree {
        let mut tree = Tree::new("C:\\test".into(), "test".into(), 0, None, 1);
        let a = add_child(&mut tree, 0, Node::new_dir("a", 0, 0, false));
        add_child(
            &mut tree,
            a,
            Node::new_file("a1.bin", 0, 300, 0, Category::Other, false),
        );
        add_child(
            &mut tree,
            a,
            Node::new_file("a2.mp4", 0, 700, 0, Category::Video, false),
        );
        let b = add_child(&mut tree, 0, Node::new_dir("b", 0, 0, false));
        add_child(
            &mut tree,
            b,
            Node::new_file("b1.txt", 0, 100, 0, Category::Document, false),
        );
        add_child(
            &mut tree,
            0,
            Node::new_file("root.dat", 0, 50, 0, Category::Other, false),
        );
        aggregate(&mut tree.nodes);
        tree
    }

    #[test]
    fn aggregation_sums_sizes_and_counts() {
        let tree = sample_tree();
        assert_eq!(tree.node(1).size, 1000);
        assert_eq!(tree.node(1).files, 2);
        assert_eq!(tree.node(4).size, 100);
        assert_eq!(tree.total_bytes(), 1150);
        assert_eq!(tree.total_files(), 4);
        assert_eq!(tree.total_dirs(), 2);
        tree.validate().unwrap();
    }

    #[test]
    fn children_sorted_desc_by_size() {
        let tree = sample_tree();
        let kids = tree.children_sorted(0);
        assert_eq!(kids, vec![1, 4, 6]);
        assert_eq!(tree.path_of(5), "C:\\test\\b\\b1.txt");
    }

    #[test]
    fn sorting_covers_all_orders_and_search_before_limit() {
        let mut tree = sample_tree();
        tree.node_mut(1).mtime = 20;
        tree.node_mut(4).mtime = 30;
        tree.node_mut(6).mtime = 10;
        for (order, expected) in [
            (SortOrder::SizeDescending, vec![1, 4, 6]),
            (SortOrder::SizeAscending, vec![6, 4, 1]),
            (SortOrder::NameAscending, vec![1, 4, 6]),
            (SortOrder::NameDescending, vec![6, 4, 1]),
            (SortOrder::ModifiedDescending, vec![4, 1, 6]),
            (SortOrder::ModifiedAscending, vec![6, 1, 4]),
        ] {
            assert_eq!(tree.children_sorted_by(0, order), expected);
        }
        assert_eq!(tree.find_sorted("1", 1, SortOrder::SizeAscending), vec![5]);
        assert_eq!(tree.find_sorted("1", 1, SortOrder::SizeDescending), vec![2]);
    }

    #[test]
    fn path_uses_root_and_names() {
        let tree = sample_tree();
        assert_eq!(tree.path_of(0), "C:\\test");
        assert_eq!(tree.path_of(5), "C:\\test\\b\\b1.txt");
    }

    #[test]
    fn drive_root_path_has_single_separator() {
        let tree = Tree::new("C:\\".into(), "C:\\".into(), 0, None, 1);
        assert_eq!(tree.path_of(0), "C:\\");
    }

    #[test]
    fn top_files_finds_largest() {
        let tree = sample_tree();
        let top = tree.top_files(0, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(tree.node(top[0]).size, 700);
        assert_eq!(tree.node(top[1]).size, 300);
    }

    #[test]
    fn type_breakdown_groups() {
        let tree = sample_tree();
        let b = tree.type_breakdown(0);
        let video = b.iter().find(|(c, _, _)| *c == Category::Video).unwrap();
        assert_eq!(video.1, 700);
        assert_eq!(video.2, 1);
        let docs = b.iter().find(|(c, _, _)| *c == Category::Document).unwrap();
        assert_eq!(docs.1, 100);
    }

    #[test]
    fn find_is_turkish_case_insensitive() {
        let mut tree = sample_tree();
        let id = add_child(
            &mut tree,
            0,
            Node::new_file("İndirilenler.zip", 0, 10, 0, Category::Archive, false),
        );
        let hits = tree.find("indir", 10);
        assert!(hits.contains(&id));
    }

    #[test]
    fn remove_subtracts_from_ancestors_and_unlinks() {
        let mut tree = sample_tree();
        let removed = tree.remove(4);
        assert_eq!(removed, Some(100));
        assert_eq!(tree.total_bytes(), 1050);
        assert_eq!(tree.total_dirs(), 1);
        assert_eq!(tree.total_files(), 3);
        assert!(!tree.children(0).collect::<Vec<_>>().contains(&4));
        tree.validate().unwrap();
    }

    #[test]
    fn cannot_remove_root() {
        let mut tree = sample_tree();
        assert_eq!(tree.remove(0), None);
    }

    #[test]
    fn ancestry_checks() {
        let tree = sample_tree();
        assert!(tree.is_ancestor(0, 5));
        assert!(tree.is_ancestor(4, 5));
        assert!(!tree.is_ancestor(1, 5));
    }
}
