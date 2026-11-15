//! Paralel dizin tarayicisi.
//!
//! Tasarim:
//! * Is parcalari (klasor, dugum kimligi) paylasilan bir yiginda tutulur.
//! * N is parcacigi ayni anda farkli klasorleri okur; arena tek kilit altinda
//!   toplu ekleme yapar (klasor basina bir kilit).
//! * Toplamlar tarama bitince tersten dolasimla tek geciste hesaplanir.
//! * Iptal edilirse o ana kadar bulunan agac "kismi" olarak dondurulur.
//!
//! Windows'ta `\\?\` oneki ile uzun yol siniri asilir; yeniden ayristirma
//! noktalarinin icine girilmez (dongu korumasi).

use crate::categories::Category;
use crate::model::{NONE, Node, Tree, aggregate};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub threads: usize,
}

impl Default for ScanOptions {
    fn default() -> Self {
        let cores = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8);
        Self {
            threads: cores.clamp(2, 16),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProgressSnapshot {
    pub files: u64,
    pub dirs: u64,
    pub bytes: u64,
    pub errors: u64,
    pub elapsed: Duration,
    pub running: bool,
    pub canceled: bool,
}

pub enum ScanStatus {
    Running(ProgressSnapshot),
    Done(Tree),
}

struct Queue {
    tasks: Vec<(u32, PathBuf)>,
    /// Kuyrukta bekleyen + su anda islenen klasor sayisi.
    pending: usize,
}

struct Shared {
    tree: Mutex<Option<Tree>>,
    queue: Mutex<Queue>,
    cv: Condvar,
    files: AtomicU64,
    dirs: AtomicU64,
    bytes: AtomicU64,
    errors: AtomicU64,
    running: AtomicBool,
    canceled: AtomicBool,
    result: Mutex<Option<Tree>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct ScanSession {
    pub root_path: String,
    pub root_display: String,
    pub started: Instant,
    pub volume_total: Option<u64>,
    pub threads: usize,
    shared: Arc<Shared>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ScanSession {
    /// Taramayi baslatir. Gecersiz yol hemen `Err` dondurur.
    pub fn start(
        input: &str,
        volume_total: Option<u64>,
        options: ScanOptions,
    ) -> Result<Self, String> {
        let display = normalize_display_path(input)?;
        let io_root = io_path(&display);
        let meta = fs::metadata(&io_root).map_err(|e| format!("Klasor acilamadi: {e}"))?;
        if !meta.is_dir() {
            return Err("Secilen yol bir klasor degil.".to_string());
        }
        let root_mtime = mtime_of(&meta);
        let root_name = display
            .trim_end_matches('\\')
            .rsplit('\\')
            .next()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| display.clone());

        let mut tree = Tree::new(
            display.clone(),
            root_name,
            root_mtime,
            volume_total,
            options.threads,
        );
        tree.nodes[0].size = 0;

        let shared = Arc::new(Shared {
            tree: Mutex::new(Some(tree)),
            queue: Mutex::new(Queue {
                tasks: vec![(0, io_root)],
                pending: 1,
            }),
            cv: Condvar::new(),
            files: AtomicU64::new(0),
            dirs: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            running: AtomicBool::new(true),
            canceled: AtomicBool::new(false),
            result: Mutex::new(None),
        });

        let started = Instant::now();
        let shared_thread = Arc::clone(&shared);
        let threads = options.threads.max(1);
        let handle = thread::spawn(move || {
            let mut workers = Vec::with_capacity(threads);
            for _ in 0..threads {
                let sh = Arc::clone(&shared_thread);
                workers.push(thread::spawn(move || worker(sh)));
            }
            for w in workers {
                let _ = w.join();
            }
            finalize(&shared_thread, started);
        });

        Ok(Self {
            root_path: display.clone(),
            root_display: display,
            started,
            volume_total,
            threads: options.threads.max(1),
            shared,
            handle: Some(handle),
        })
    }

    pub fn progress(&self) -> ProgressSnapshot {
        let running = self.shared.running.load(Ordering::Relaxed);
        ProgressSnapshot {
            files: self.shared.files.load(Ordering::Relaxed),
            dirs: self.shared.dirs.load(Ordering::Relaxed),
            bytes: self.shared.bytes.load(Ordering::Relaxed),
            errors: self.shared.errors.load(Ordering::Relaxed),
            elapsed: self.started.elapsed(),
            running,
            canceled: self.shared.canceled.load(Ordering::Relaxed),
        }
    }

    pub fn is_running(&self) -> bool {
        self.shared.running.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.shared.canceled.store(true, Ordering::SeqCst);
        self.shared.cv.notify_all();
    }

    pub fn was_canceled(&self) -> bool {
        self.shared.canceled.load(Ordering::Relaxed)
    }

    /// Tarama bittiyse sonucu alir (yalnizca bir kez).
    pub fn try_finish(&mut self) -> Option<Tree> {
        if let Some(tree) = lock(&self.shared.result).take() {
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
            return Some(tree);
        }
        None
    }
    /// Bitene kadar bekler (testler icin).
    pub fn wait(mut self) -> Option<Tree> {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        lock(&self.shared.result).take()
    }
}

impl Drop for ScanSession {
    fn drop(&mut self) {
        // Pencere kapanirken tarama arka planda asili kalmasin.
        self.cancel();
    }
}

struct RawEntry {
    name: OsString,
    size: u64,
    is_dir: bool,
    mtime: i64,
    category: Category,
    reparse: bool,
}

fn worker(shared: Arc<Shared>) {
    loop {
        let (parent_id, dir_path) = {
            let mut queue = lock(&shared.queue);
            loop {
                if shared.canceled.load(Ordering::Relaxed) {
                    return;
                }
                if let Some(task) = queue.tasks.pop() {
                    break task;
                }
                if queue.pending == 0 {
                    return;
                }
                queue = shared
                    .cv
                    .wait(queue)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        };

        let (entries, errors) = read_directory(&dir_path);
        if errors > 0 {
            shared.errors.fetch_add(errors, Ordering::Relaxed);
        }

        let mut file_count = 0u64;
        let mut dir_count = 0u64;
        let mut byte_count = 0u64;
        for e in &entries {
            if e.is_dir {
                dir_count += 1;
            } else {
                file_count += 1;
                byte_count += e.size;
            }
        }

        let subdirs = insert_entries(&shared, parent_id, entries);

        {
            let mut queue = lock(&shared.queue);
            queue.pending += subdirs.len();
            for (id, name) in &subdirs {
                queue.tasks.push((*id, dir_path.join(name)));
            }
        }
        shared.cv.notify_all();

        shared.files.fetch_add(file_count, Ordering::Relaxed);
        shared.dirs.fetch_add(dir_count, Ordering::Relaxed);
        shared.bytes.fetch_add(byte_count, Ordering::Relaxed);

        // Bu klasor bitti.
        let mut queue = lock(&shared.queue);
        queue.pending -= 1;
        if queue.pending == 0 {
            shared.cv.notify_all();
        }
    }
}

fn read_directory(path: &Path) -> (Vec<RawEntry>, u64) {
    let mut entries = Vec::new();
    let mut errors = 0;
    let Ok(reader) = fs::read_dir(path) else {
        return (entries, 1);
    };
    for item in reader {
        let Ok(entry) = item else {
            errors += 1;
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            errors += 1;
            continue;
        };
        let is_dir = file_type.is_dir();
        let (size, mtime, reparse) = match entry.metadata() {
            Ok(meta) => {
                let size = if is_dir { 0 } else { meta.len() };
                (size, mtime_of(&meta), is_reparse(&meta))
            }
            Err(_) => {
                errors += 1;
                (0, 0, false)
            }
        };
        let name = entry.file_name();
        let category = if is_dir {
            Category::Other
        } else if let Some(text) = name.to_str() {
            Category::from_file_name(text)
        } else {
            Category::from_file_name(&name.to_string_lossy())
        };
        entries.push(RawEntry {
            name,
            size,
            is_dir,
            mtime,
            category,
            reparse,
        });
    }
    (entries, errors)
}

fn insert_entries(shared: &Shared, parent_id: u32, entries: Vec<RawEntry>) -> Vec<(u32, OsString)> {
    let mut guard = lock(&shared.tree);
    let Some(tree) = guard.as_mut() else {
        return Vec::new();
    };
    let mut subdirs = Vec::new();
    for raw in entries {
        let id = tree.nodes.len() as u32;
        let display_name = raw.name.to_string_lossy().into_owned();
        let node = if raw.is_dir {
            Node::new_dir(display_name, parent_id, raw.mtime, raw.reparse)
        } else {
            Node::new_file(
                display_name,
                parent_id,
                raw.size,
                raw.mtime,
                raw.category,
                raw.reparse,
            )
        };
        let prev_last = tree.nodes[parent_id as usize].last_child;
        tree.nodes.push(node);
        if prev_last != NONE {
            tree.nodes[prev_last as usize].next_sibling = id;
        }
        let parent = &mut tree.nodes[parent_id as usize];
        if parent.first_child == NONE {
            parent.first_child = id;
        }
        parent.last_child = id;
        if raw.is_dir && !raw.reparse {
            subdirs.push((id, raw.name));
        }
    }
    subdirs
}

fn finalize(shared: &Shared, started: Instant) {
    let tree = {
        let mut guard = lock(&shared.tree);
        guard.take()
    };
    if let Some(mut tree) = tree {
        aggregate(&mut tree.nodes);
        tree.errors = shared.errors.load(Ordering::Relaxed);
        tree.elapsed = started.elapsed();
        tree.partial = shared.canceled.load(Ordering::Relaxed);
        *lock(&shared.result) = Some(tree);
    }
    shared.running.store(false, Ordering::Relaxed);
    shared.cv.notify_all();
}

#[cfg(windows)]
fn is_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse(meta: &fs::Metadata) -> bool {
    meta.file_type().is_symlink()
}

fn mtime_of(meta: &fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Girdiyi temizler: tirnaklar, egik cizgiler, sondaki ayirici.
pub fn normalize_display_path(input: &str) -> Result<String, String> {
    let mut s = input.trim().trim_matches('"').trim().replace('/', "\\");
    if s.is_empty() {
        return Err("Bir klasor yolu girin.".to_string());
    }
    // `C:` -> `C:\`
    if s.len() == 2 && s.ends_with(':') {
        s.push('\\');
    }
    // Surucu koku disinda sondaki ters bolu kaldirilir.
    if s.len() > 3 && s.ends_with('\\') {
        while s.len() > 3 && s.ends_with('\\') {
            s.pop();
        }
    }
    // Gecerli mi?
    let p = Path::new(&s);
    if !p.is_absolute() {
        return Err("Tam bir yol girin (ornek: C:\\Users).".to_string());
    }
    Ok(s)
}

/// Win32 uzun yol destegi icin `\\?\` oneki.
pub fn io_path(display: &str) -> PathBuf {
    if display.starts_with(r"\\?\") {
        PathBuf::from(display)
    } else if let Some(rest) = display.strip_prefix(r"\\") {
        PathBuf::from(format!(r"\\?\UNC\{rest}"))
    } else {
        PathBuf::from(format!(r"\\?\{display}"))
    }
}

/// Bir yolun surucu koku olup olmadigini soyler (`C:\` gibi).
pub fn is_drive_root(path: &str) -> bool {
    let b = path.as_bytes();
    (b.len() == 3 && b[1] == b':' && b[2] == b'\\') || (b.len() == 2 && b[1] == b':')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct TempTree {
        root: PathBuf,
    }

    impl TempTree {
        fn new(tag: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "iskandil-test-{tag}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn file(&self, rel: &str, bytes: usize) {
            let p = self.root.join(rel);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let mut f = fs::File::create(&p).unwrap();
            f.write_all(&vec![7u8; bytes]).unwrap();
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn scans_sizes_and_counts() {
        let t = TempTree::new("scan");
        t.file("a.bin", 100);
        t.file("sub/b.mp4", 500);
        t.file("sub/deep/c.txt", 200);
        t.file("empty.dat", 0);
        fs::create_dir_all(t.root.join("bos")).unwrap();

        let session =
            ScanSession::start(t.root.to_str().unwrap(), None, ScanOptions { threads: 4 }).unwrap();
        let tree = session.wait().expect("sonuc");
        assert_eq!(tree.total_bytes(), 800);
        assert_eq!(tree.total_files(), 4);
        assert_eq!(tree.total_dirs(), 3);
        assert!(!tree.partial);
        assert_eq!(
            tree.path_of(0),
            normalize_display_path(t.root.to_str().unwrap()).unwrap()
        );
        let top = tree.top_files(tree.root, 1);
        assert_eq!(tree.node(top[0]).size, 500);
        tree.validate().unwrap();
    }

    #[test]
    fn normalizes_paths() {
        assert_eq!(
            normalize_display_path("  \"C:/Users/Kayon/\" ").unwrap(),
            "C:\\Users\\Kayon"
        );
        assert_eq!(normalize_display_path("C:").unwrap(), "C:\\");
        assert_eq!(normalize_display_path("C:\\").unwrap(), "C:\\");
        assert!(normalize_display_path("goreli\\yol").is_err());
        assert!(normalize_display_path("").is_err());
    }

    #[test]
    fn io_prefix_is_well_formed() {
        assert_eq!(io_path("C:\\Users").to_string_lossy(), r"\\?\C:\Users");
        assert_eq!(
            io_path(r"\\sunucu\pay").to_string_lossy(),
            r"\\?\UNC\sunucu\pay"
        );
        assert_eq!(io_path(r"\\?\C:\x").to_string_lossy(), r"\\?\C:\x");
    }

    #[test]
    fn drive_root_detection() {
        assert!(is_drive_root("C:\\"));
        assert!(!is_drive_root("C:\\Users"));
    }
}
