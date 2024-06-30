use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();
    let vcpkg_root = env::var("VCPKG_ROOT").expect("VCPKG_ROOT environment variable is not set");
    let package_path = PathBuf::from(&vcpkg_root).join("packages/libvpx_x64-windows-static");
    let lib_dir = package_path.join("lib");
    let include_dir = package_path.join("include");
    let control_file = package_path.join("CONTROL");

    let vpx_version = read_version_from_control(&control_file).expect("Failed to read VPX version from control file");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=vpx");
    println!("cargo:include={}", include_dir.display());
    println!("cargo:rustc-env=VPX_VERSION={}", vpx_version);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", control_file.display());
}

fn read_version_from_control(control_file: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(control_file).ok()?;
    for line in content.lines() {
        if line.starts_with("Version:") {
            return Some(line["Version:".len()..].trim().to_string());
        }
    }
    None
}
