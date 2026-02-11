#![forbid(unsafe_code)]

pub mod ph1_voice_id;
pub mod ph1capreq;
pub mod ph1explain;
pub mod ph1l;
pub mod ph1link;
pub mod ph1onb;
pub mod ph1position;
pub mod ph1w;
pub mod ph1x;
pub mod simulation_executor;

pub fn hello_compile() -> &'static str {
    "hello compile"
}
