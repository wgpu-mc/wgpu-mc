use std::path::PathBuf;

use cargo_lock::Lockfile;

fn main() {
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set")
        .into();

    let lockfile_path = manifest_dir
        .ancestors()
        .find_map(|p| {
            let lockfile = p.join("Cargo.lock");
            lockfile.exists().then(|| lockfile.to_owned())
        })
        .expect("Could not find Cargo.lock");

    let lockfile = Lockfile::load(lockfile_path).unwrap();

    let wgpu = lockfile
        .packages
        .iter()
        .find(|p| p.name.as_str() == "wgpu")
        .expect("Could not find wgpu in Cargo.lock");

    println!("cargo::rustc-env=WGPUMC_WGPU_VER={}", wgpu.version)
}
