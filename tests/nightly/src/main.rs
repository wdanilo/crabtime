// === Rust Nightly Test ===

#[crabtime::function]
fn gen_positions(components: Vec<String>) {
    for (ix, name) in components.iter().enumerate() {
        let dim = ix + 1;
        let cons = components[0..dim].join(",");
        crabtime::output! {
            enum Position{{dim}} {
                {{cons}}
            }
        }
    }
}
gen_positions!(["X", "Y", "Z", "W"]);

#[crabtime::function]
fn gen_paths() {
    let workspace_path = format!("\"{}\"", crate::crabtime::WORKSPACE_PATH);
    let crate_config_path = format!("\"{}\"", crate::crabtime::CRATE_CONFIG_PATH);
    let call_site_file_path = format!("\"{}\"", crate::crabtime::CALL_SITE_FILE_PATH);

    crabtime::output! {
        const WORKSPACE_PATH: &str = {{workspace_path}};
        const CRATE_ROOT: &str = {{crate_config_path}};
        const CALL_SITE_FILE_PATH: &str = {{call_site_file_path}};
    }
}
gen_paths!();

fn main() {
    let _p1 = Position2::X;
    let workspace_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(WORKSPACE_PATH, workspace_path);
    assert_eq!(CRATE_ROOT, format!("{workspace_path}/tests/nightly"));
    assert_eq!(
        CALL_SITE_FILE_PATH,
        format!("{workspace_path}/tests/nightly/src/main.rs")
    );
}
