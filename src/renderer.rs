use ash::vk;

pub mod vulkan_app;
#[cfg(debug_assertions)]
mod debug_utils;
mod device_selection;

pub const WINDOW_TITLE: &str = "AL-Engine";
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub const ENGINE_VERSION: u32 = vk::make_version(0, 1, 0);
pub const API_VERSION: u32 = vk::make_version(1, 0, 92);
