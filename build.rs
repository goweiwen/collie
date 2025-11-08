use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");

    // Check if frontend/dist exists, if not, build it
    let dist_exists = std::path::Path::new("frontend/dist").exists();

    if !dist_exists {
        println!("cargo:warning=Frontend dist folder not found, building frontend...");
        build_frontend();
    }
}

fn build_frontend() {
    // Install dependencies if node_modules doesn't exist
    if !std::path::Path::new("frontend/node_modules").exists() {
        println!("cargo:warning=Installing frontend dependencies...");
        let status = Command::new("npm")
            .args(["install"])
            .current_dir("frontend")
            .status()
            .expect("Failed to install frontend dependencies");

        if !status.success() {
            panic!("Failed to install frontend dependencies");
        }
    }

    // Build frontend
    println!("cargo:warning=Building frontend...");
    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("Failed to build frontend");

    if !status.success() {
        panic!("Failed to build frontend");
    }
}
