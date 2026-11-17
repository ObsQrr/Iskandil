fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/iskandil.ico");
        res.set("ProductName", "İskandil");
        res.set("FileDescription", "İskandil — disk derinlik ölçer");
        res.set("OriginalFilename", "iskandil.exe");
        res.set("LegalCopyright", "MIT");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        if let Err(error) = res.compile() {
            println!("cargo:warning=Windows kaynak dosyasi derlenemedi: {error}");
        }
    }
}
