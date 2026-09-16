use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let ico = manifest_dir.join("assets").join("appicon.ico");
    println!("cargo:rerun-if-changed={}", ico.display());

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if !ico.exists() {
        println!(
            "cargo:warning=assets/appicon.ico not found — \
             .exe will have no file icon. Run `just win-build` to generate it."
        );
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico.to_str().unwrap());
    res.compile()
        .expect("build.rs: failed to compile Windows icon resource");
}
