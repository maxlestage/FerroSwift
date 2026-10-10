//! The FerroSwift website: what the server and the browser share (routes, the syntax
//! highlighter, the examples and the playground island), the static generator and
//! development server (native) and the browser bundle (wasm32).

pub mod examples;
pub mod highlight;
pub mod islands;
pub mod labels;
pub mod routes;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(target_arch = "wasm32")]
mod client;
