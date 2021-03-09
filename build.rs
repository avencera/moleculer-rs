fn main() {
    let mut opts = built::Options::default();
    opts.set_ci(false);
    opts.set_env(false);
    opts.set_dependencies(false);
    opts.set_features(false);
    opts.set_cfg(false);

    let src = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("built.rs");

    built::write_built_file_with_opts(&opts, src.as_ref(), &dst)
        .expect("Failed to acquire build-time information");
}
