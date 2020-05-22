use ash::vk;

mod buffers;
#[cfg(feature = "validation-layers")]
mod debug_utils;
mod device_selection;
mod graphics_pipeline;
mod swapchain;
mod sync;
mod vertex;
pub mod vulkan_app;

pub const WINDOW_TITLE: &str = "AL-Engine";
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub const ENGINE_VERSION: u32 = vk::make_version(0, 1, 0);
pub const VULKAN_VERSION: u32 = vk::make_version(1, 0, 92);

pub(self) const REQUIRED_DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];
pub(self) const SHADERS_LOCATION: [&str; 2] = [".", "shaders"];

pub(self) const MAX_FRAMES_IN_FLIGHT: usize = 2;
