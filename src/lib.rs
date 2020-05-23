use ash::vk;

pub mod fps_limiter;
pub mod renderer;
pub mod utils;

pub const APPLICATION_VERSION: u32 = vk::make_version(0, 1, 0);
pub const FPS_LIMIT: f32 = 60f32;
