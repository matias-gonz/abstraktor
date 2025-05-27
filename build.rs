use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let llvm_dir = Path::new(&manifest_dir).join("llvm");

    println!("cargo:rerun-if-changed=llvm/");
    println!("cargo:rerun-if-changed=llvm/Makefile");
    println!("cargo:rerun-if-changed=llvm/src/");

    if !llvm_dir.exists() {
        panic!("LLVM directory not found at: {}", llvm_dir.display());
    }

    let makefile_path = llvm_dir.join("Makefile");
    if !makefile_path.exists() {
        panic!("Makefile not found at: {}", makefile_path.display());
    }

    let output = Command::new("make")
        .current_dir(&llvm_dir)
        .arg("all")
        .output()
        .expect(
            "Failed to execute make command. Make sure 'make' is installed and available in PATH.",
        );

    if !output.status.success() {
        eprintln!("Make stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Make stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to build LLVM components with make");
    }
}
