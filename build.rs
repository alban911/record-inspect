use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);
    let assets_dir = manifest_path.join("assets");
    let rc_file = assets_dir.join("app.rc");
    let out_obj = out_dir.join("app_icon.o");

    if rc_file.exists() {
        let _ = Command::new("windres")
            .current_dir(&assets_dir)
            .args(&["-i", "app.rc", "-o"])
            .arg(&out_obj)
            .status();
    }

    if !out_obj.exists() {
        let existing_obj = assets_dir.join("app_icon.o");
        if existing_obj.exists() {
            let _ = fs::copy(&existing_obj, &out_obj);
        }
    }

    if out_obj.exists() {
        println!("cargo:rustc-link-arg={}", out_obj.display());
    }

    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app_icon.ico");
}
