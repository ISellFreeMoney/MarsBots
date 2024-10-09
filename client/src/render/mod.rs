//! Rendering part of the client

/* WebGPU HELPER MODULES */
mod buffers;
mod init;
mod render;
pub use self::render::{buffer_from_slice, clear_color_and_depth, clear_depth, to_u8_slice};

/* OTHER HELPER MODULES */
mod frustum;
pub use self::frustum::Frustum;

/* RENDERING-RESPONSIBLE MODULES */
mod ui;
pub mod world;
pub use self::ui::UiRenderer;
pub use self::world::{Model, WorldRenderer};
