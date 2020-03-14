use log::trace;
use std::sync::Arc;
use vulkano::device::DeviceExtensions;
use vulkano::instance::{Instance, PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::swapchain::Surface;
use winit::window::Window;

pub struct QueueFamilyId {
    pub graphics: u32,
    pub presentation: u32,
}

impl QueueFamilyId {
    pub fn new(graphics: u32, presentation: u32) -> Self {
        Self {
            graphics,
            presentation,
        }
    }
}

struct QueueFamilyIdBuilder {
    graphics: Option<u32>,
    graphics_score: usize,
    presentation: Option<u32>,
    presentation_score: usize,
}

impl QueueFamilyIdBuilder {
    pub fn new() -> Self {
        QueueFamilyIdBuilder {
            graphics: None,
            graphics_score: 0,
            presentation: None,
            presentation_score: 0,
        }
    }

    fn rank_queue(family: &QueueFamily) -> usize {
        let mut score = 0;
        // Increase score for each bonus queue
        score += (family.queues_count() - 1) * 100;

        // If case of a tie, this will make the difference
        if let Some(supported_bits) = family.timestamp_valid_bits() {
            // Between 36..64
            score += supported_bits as usize;
        }

        score
    }

    pub fn try_set_graphics(&mut self, family: &QueueFamily) {
        let score = Self::rank_queue(family);
        if score > self.graphics_score {
            self.graphics_score = score;
            self.graphics = Some(family.id());
        }
    }

    pub fn try_set_presentation(&mut self, family: &QueueFamily) {
        let score = Self::rank_queue(family);
        if score > self.presentation_score {
            self.presentation_score = score;
            self.presentation = Some(family.id());
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.presentation.is_some()
    }
}

impl Into<QueueFamilyId> for QueueFamilyIdBuilder {
    fn into(self) -> QueueFamilyId {
        QueueFamilyId::new(self.graphics.unwrap(), self.presentation.unwrap())
    }
}

pub fn pick_physical_device<'a>(
    instance: &'a Arc<Instance>,
    surface: &Arc<Surface<Window>>,
) -> PhysicalDevice<'a> {
    PhysicalDevice::enumerate(&instance)
        .filter(|device| {
            trace!("Trying device: {:?}", device.name());
            is_device_suitable(surface, &device)
        })
        .map(|device| {
            let score = score_device(&device);
            trace!("Device {:?} scored {}", device.name(), score);
            (device, score)
        })
        .max_by_key(|(_, score)| *score)
        .expect("Failed to find a suitable GPU !")
        .0
}

fn is_device_suitable(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> bool {
    let families = find_queue_families(surface, device);
    let extensions_supported = check_device_extension_support(device);

    let swap_chain_adequate = if extensions_supported {
        let capabilities = surface
            .capabilities(*device)
            .expect("Failed to get surface capabilities");

        !capabilities.supported_formats.is_empty()
            && capabilities.present_modes.iter().next().is_some()
    } else {
        false
    };

    families.is_some() && extensions_supported && swap_chain_adequate
}

fn score_device(device: &PhysicalDevice) -> u32 {
    let mut score = 0;

    // Prefer dedicated GPU over everything else
    score += match device.ty() {
        PhysicalDeviceType::DiscreteGpu => 1_000,
        PhysicalDeviceType::VirtualGpu => 500,
        PhysicalDeviceType::IntegratedGpu => 100,
        _ => 0,
    };

    // In case of a tie, choose based on the amount of memory available
    let memory = device
        .memory_heaps()
        .filter(|heap| heap.is_device_local())
        .map(|heap| heap.size())
        .sum::<usize>();

    // Add to the score but in Go instead of bytes
    score += (memory / 1_000_000_000) as u32;

    score
}

pub fn find_queue_families(
    surface: &Arc<Surface<Window>>,
    device: &PhysicalDevice,
) -> Option<QueueFamilyId> {
    let mut families_id = QueueFamilyIdBuilder::new();

    for queue_family in device.queue_families() {
        if queue_family.supports_graphics() {
            families_id.try_set_graphics(&queue_family);
        }

        if surface.is_supported(queue_family).unwrap() {
            families_id.try_set_presentation(&queue_family)
        }

        if families_id.is_complete() {
            break;
        }
    }

    if families_id.is_complete() {
        Some(families_id.into())
    } else {
        None
    }
}

fn check_device_extension_support(device: &PhysicalDevice) -> bool {
    let available_extensions: DeviceExtensions = DeviceExtensions::supported_by_device(*device);
    let required_extensions = required_extensions();

    available_extensions.intersection(&required_extensions) == required_extensions
}

/// Cannot be converted to a const because of [DeviceExtensions::none()](DeviceExtensions::none)
pub fn required_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    }
}
