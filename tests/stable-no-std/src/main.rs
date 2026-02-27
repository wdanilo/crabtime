#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { libc::exit(1) }
}

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

#[unsafe(no_mangle)]
fn main() {
    let _p1 = Position2::X;
    let path = WORKSPACE_PATH.as_bytes();
    let prefix = env!("CARGO_MANIFEST_DIR").as_bytes();
    let mut index = 0;
    while index < path.len() {
        assert!(path[index] == prefix[index]);
        index += 1;
    }

    unsafe { libc::exit(0) }
}
