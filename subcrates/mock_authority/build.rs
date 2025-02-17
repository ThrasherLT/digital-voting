use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

fn main() {
    let frontend_path = Path::new("frontend");
    let out_dir = PathBuf::from_str(&env::var("OUT_DIR").expect("Cargo should set OUT_DIR"))
        .expect("OUT_DIR to be a path");
    let target_dir = out_dir.ancestors().nth(3).expect("Path ancestor to exist");
    let dist_dir = target_dir.join("mock_authority_frontend");
    fs::create_dir_all(dist_dir.clone()).expect("dist_dir to be created");

    println!("cargo:rerun-if-changed=frontend/src/");
    println!("cargo:rerun-if-changed=frontend/index.html");

    let status = Command::new("trunk")
        .args(["build", "--release", "--dist", dist_dir.to_str().unwrap()])
        .current_dir(frontend_path)
        .status()
        .expect("Failed to run trunk build");

    if !status.success() {
        panic!("Trunk build failed!");
    }
}
