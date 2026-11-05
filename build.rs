use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);
    let assets_dir = manifest_path.join("assets");
    let rc_file = assets_dir.join("app.rc");
    let obj_file = assets_dir.join("app_icon.o");

    if rc_file.exists() {
        let _ = Command::new("windres")
            .current_dir(&assets_dir)
            .args(&["-i", "app.rc", "-o", "app_icon.o"])
            .status();
    }

    if obj_file.exists() {
        println!("cargo:rustc-link-arg={}", obj_file.display());
    }
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app_icon.ico");
}
