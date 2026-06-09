use cbindgen::{Config, Language};
use std::env;
use std::panic::catch_unwind;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(Config::from_file("cbindgen.toml").unwrap())
        .with_std_types(false)
        .with_cpp_compat(false)
        .with_language(Language::C)
        .with_parse_deps(true)
        .with_parse_include(&["wgpu"])
        .exclude_item("FfiStr")
        // .exclude_item("Arc<Surface>")
        .exclude_item("Sampler")
        .exclude_item("Vec")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bindings.h");
}
