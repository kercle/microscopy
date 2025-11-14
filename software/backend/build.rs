use std::{path::Path, process::Command};

fn main() {
    let frontend_dir = Path::new("../frontend");

    let profile = std::env::var("PROFILE").unwrap();

    if profile == "release" {
        println!("cargo:rerun-if-changed=../frontend/");

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
