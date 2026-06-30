pub fn hello() -> &'static str {
    "Hello from the Rust core"
}

pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

include!(concat!(env!("OUT_DIR"), "/java_bridge.rs"));
