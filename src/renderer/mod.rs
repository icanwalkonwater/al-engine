mod device_selection;
pub mod vulkan_app;

pub use vulkan_app::VulkanApplication;

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_LUNARG_standard_validation"];

const DIMENSIONS: (u32, u32) = (800, 600);

const APPLICATION_NAME: &str = "AL-Engine";
