//! Dosya turleri. Renkler arayuz katmaninda tanimlanir; cekirdek yalnizca siniflandirir.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Category {
    Video = 0,
    Image = 1,
    Audio = 2,
    Document = 3,
    Code = 4,
    Archive = 5,
    App = 6,
    Other = 7,
}

pub const ALL: [Category; 8] = [
    Category::Video,
    Category::Image,
    Category::Audio,
    Category::Document,
    Category::Code,
    Category::Archive,
    Category::App,
    Category::Other,
];

impl Category {
    pub const COUNT: usize = 8;
    pub const ALL: [Category; 8] = crate::categories::ALL;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label_tr(self) -> &'static str {
        match self {
            Category::Video => "Video",
            Category::Image => "Gorsel",
            Category::Audio => "Ses",
            Category::Document => "Belge",
            Category::Code => "Kod",
            Category::Archive => "Arsiv",
            Category::App => "Uygulama",
            Category::Other => "Diger",
        }
    }

    pub fn from_file_name(name: &str) -> Self {
        let Some(ext) = extension_of(name) else {
            return Category::Other;
        };
        // Kucuk harfe cevirirken heap ayirmadan sabit tampon kullan.
        let mut buf = [0u8; 12];
        let bytes = ext.as_bytes();
        if bytes.len() > buf.len() {
            return Category::Other;
        }
        for (dst, src) in buf.iter_mut().zip(bytes) {
            *dst = src.to_ascii_lowercase();
        }
        let Ok(ext) = std::str::from_utf8(&buf[..bytes.len()]) else {
            return Category::Other;
        };
        match ext {
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
            | "m2ts" | "vob" | "3gp" | "ogv" => Category::Video,
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tif" | "tiff" | "heic" | "heif"
            | "svg" | "ico" | "psd" | "raw" | "cr2" | "cr3" | "nef" | "arw" | "dng" | "avif" => {
                Category::Image
            }
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "aiff" | "aif"
            | "mid" | "midi" => Category::Audio,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt"
            | "ods" | "odp" | "csv" | "md" | "epub" | "mobi" | "pages" | "numbers" | "key"
            | "tex" | "log" => Category::Document,
            "rs" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "py" | "pyw" | "js" | "mjs"
            | "cjs" | "ts" | "tsx" | "jsx" | "java" | "kt" | "kts" | "go" | "cs" | "php" | "rb"
            | "swift" | "dart" | "lua" | "pl" | "r" | "scala" | "clj" | "ex" | "exs" | "hs"
            | "asm" | "s" | "html" | "htm" | "css" | "scss" | "sass" | "less" | "json" | "xml"
            | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "sh" | "bash" | "zsh" | "ps1"
            | "bat" | "cmd" | "sql" | "vue" | "svelte" | "glsl" | "wgsl" | "vert" | "frag"
            | "proto" | "gradle" | "cmake" | "make" | "mk" => Category::Code,
            "zip" | "rar" | "7z" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "zst" | "iso" | "cab"
            | "lz" | "lzma" | "jar" | "war" | "apk" | "ipa" | "nupkg" | "deb" | "rpm" => {
                Category::Archive
            }
            "exe" | "dll" | "msi" | "msp" | "sys" | "so" | "dylib" | "bin" | "appx" | "msix"
            | "com" | "scr" | "drv" | "ocx" | "lib" | "a" | "o" | "obj" | "pdb" | "node"
            | "rlib" | "rmeta" => Category::App,
            _ => Category::Other,
        }
    }
}

/// Uzantiyi kucuk harfe cevirip dondurur. `.` ile baslayan gizli dosyalarda uzanti yok sayilir.
pub fn extension_of(name: &str) -> Option<&str> {
    let name = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let dot = name.rfind('.')?;
    if dot == 0 || dot + 1 >= name.len() {
        return None;
    }
    let ext = &name[dot + 1..];
    if ext.len() > 12 {
        return None;
    }
    Some(ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_are_classified() {
        assert_eq!(Category::from_file_name("klip.MP4"), Category::Video);
        assert_eq!(Category::from_file_name("foto.jpeg"), Category::Image);
        assert_eq!(Category::from_file_name("ses.flac"), Category::Audio);
        assert_eq!(Category::from_file_name("rapor.pdf"), Category::Document);
        assert_eq!(Category::from_file_name("main.RS"), Category::Code);
        assert_eq!(Category::from_file_name("yedek.7z"), Category::Archive);
        assert_eq!(Category::from_file_name("setup.exe"), Category::App);
        assert_eq!(Category::from_file_name("dosya"), Category::Other);
        assert_eq!(Category::from_file_name(".gitignore"), Category::Other);
        assert_eq!(Category::from_file_name("a.tar.gz"), Category::Archive);
    }

    #[test]
    fn extension_uses_last_dot() {
        assert_eq!(extension_of("C:\\a\\b.iso"), Some("iso"));
        assert_eq!(extension_of("noext"), None);
        assert_eq!(extension_of("trailing."), None);
    }
}
