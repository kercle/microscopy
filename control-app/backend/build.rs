use std::{process::Command, path::Path};

fn main() {
    let frontend_dir = Path::new("../frontend");

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
