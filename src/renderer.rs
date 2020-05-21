use ash::vk;

#[cfg(debug_assertions)]
mod debug_utils;
mod device_selection;
mod swapchain;
pub mod vulkan_app;

pub const WINDOW_TITLE: &str = "AL-Engine";
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub const ENGINE_VERSION: u32 = vk::make_version(0, 1, 0);
pub const VULKAN_VERSION: u32 = vk::make_version(1, 0, 92);

pub const REQUIRED_DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];
