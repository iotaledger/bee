fn main() {
    let ac = autocfg::new();
    ac.emit_has_type("i128");

    autocfg::rerun_path("build.rs");
}
