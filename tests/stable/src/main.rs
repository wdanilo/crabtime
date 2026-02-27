// === Rust Stable Test ===

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
    crabtime::output! {
        const WORKSPACE_PATH: &str = {{workspace_path}};
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
}
