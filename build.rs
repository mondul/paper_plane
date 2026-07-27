fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/paper_plane.ico");
        res.set("FileDescription", "Protector de pantalla de aviones de papel");
        res.set("ProductName", "paper_plane");
        res.compile().expect("no se pudo compilar el recurso de icono");
        println!("cargo:rerun-if-changed=assets/paper_plane.ico");
    }
}
