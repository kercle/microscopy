use std::{path::Path, process::Command};

fn main() {
    let frontend_dir = Path::new("../frontend");
    let communication_dir = Path::new("../communication");

    let profile = std::env::var("PROFILE").unwrap();

    if profile == "release" {
        println!("cargo:rerun-if-changed=../frontend/");

        let status = Command::new("cargo")
            .arg("test")
            .arg("export_bindings")
            .env(
                "TS_RS_EXPORT_DIR",
                format!("{}/src/lib/bindings", frontend_dir.display()),
            )
            .current_dir(communication_dir)
            .status()
            .expect("failed to run cargo test export_bindings");
    
        if !status.success() {
            panic!("exporting TypeScript bindings failed");
        }

        let status = Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(frontend_dir)
            .status()
            .expect("failed to run npm build");

        if !status.success() {
            panic!("npm build failed");
        }
    }
}
