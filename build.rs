fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/heico.ico");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icone non embarquee : {e}");
        }
    }
}
