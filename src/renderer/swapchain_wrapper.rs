use crate::renderer::DIMENSIONS;
use log::warn;
use std::sync::Arc;
use vulkano::device::{Device, DeviceOwned, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::single_pass_renderpass;
use vulkano::swapchain::{
    Capabilities, ColorSpace, CompositeAlpha, FullscreenExclusive, PresentMode,
    SupportedPresentModes, Surface, Swapchain,
};
use vulkano::sync::SharingMode;
use winit::window::Window;

pub struct SwapChainWrapper {
    swap_chain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
}

impl SwapChainWrapper {
    pub fn create(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device: usize,
        device: &Arc<Device>,
        graphics_queue: &Arc<Queue>,
        presentation_queue: &Arc<Queue>,
    ) -> Self {
        let physical_device = PhysicalDevice::from_index(&instance, physical_device).unwrap();
        let capabilities = surface
            .capabilities(physical_device)
            .expect("Failed to get surface capabilities !");

        // Screen related config
        let (surface_format, surface_color_space) =
            Self::choose_surface_format(&capabilities.supported_formats);
        let presentation_mode = Self::choose_presentation_mode(capabilities.present_modes);
        let extent = Self::choose_extent(&capabilities);

        // Amount of images in the swap chain
        // Don't remember what or why, but the +1 improves something
        let image_count = match capabilities.max_image_count {
            Some(max_image_count) if capabilities.min_image_count + 1 > max_image_count => {
                max_image_count
            }
            _ => capabilities.min_image_count + 1,
        };

        // How the images or going to be used
        let image_usage = ImageUsage {
            color_attachment: true,
            ..ImageUsage::none()
        };

        // Optimisation: if the queues are the same, don't bother setting the concurrency-safety thingy
        let sharing: SharingMode = if graphics_queue.is_same(presentation_queue) {
            graphics_queue.into()
        } else {
            vec![graphics_queue, presentation_queue].as_slice().into()
        };

        let (swap_chain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            image_count,
            surface_format,
            extent,
            1,
            image_usage,
            sharing,
            // Don't rotate or mirror anything
            capabilities.current_transform,
            // Opaque window
            CompositeAlpha::Opaque,
            presentation_mode,
            // Don't really care
            FullscreenExclusive::Default,
            // Don't render parts of the viewport out of the screen
            true,
            surface_color_space,
        )
        .expect("Failed to create swap chain !");

        Self {
            swap_chain,
            images,
            render_pass: Self::create_render_pass(device, surface_format),
        }
    }

    fn create_render_pass(
        device: &Arc<Device>,
        color_format: Format,
    ) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        Arc::new(
            single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: color_format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        )
    }

    #[inline]
    fn choose_surface_format(available_formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
        // Always choose B8G8R8A8Unorm and SrgbNonLinear or fallback to whatever is available
        // TODO improve this
        *available_formats.iter()
            .find(|(format, color_space)| {
                *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
            })
            .unwrap_or_else(|| {
                let format = &available_formats[0];
                warn!("Can't find surface format B8G8R8A8Unorm and SrgbNonLinear, falling back to {:?} and {:?}", format.0, format.1);
                format
            })
    }

    #[inline]
    fn choose_presentation_mode(
        available_presentation_modes: SupportedPresentModes,
    ) -> PresentMode {
        if available_presentation_modes.mailbox {
            PresentMode::Mailbox
        } else if available_presentation_modes.immediate {
            warn!("Presentation mode *Mailbox* not available, choosing *Immediate*");
            PresentMode::Immediate
        } else {
            // Always available
            warn!(
                "Presentation mode *Mailbox* and *Immediate* not available, falling back to *Fifo*"
            );
            PresentMode::Fifo
        }
    }

    #[inline]
    fn choose_extent(capabilities: &Capabilities) -> [u32; 2] {
        if let Some(current_extent) = capabilities.current_extent {
            current_extent
        } else {
            let mut actual_extent = [DIMENSIONS.0, DIMENSIONS.1];
            // Clamp between the min and max extents
            actual_extent[0] = capabilities.min_image_extent[0]
                .max(capabilities.max_image_extent[0])
                .min(actual_extent[0]);
            actual_extent[1] = capabilities.min_image_extent[1]
                .max(capabilities.max_image_extent[1])
                .min(actual_extent[1]);

            actual_extent
        }
    }

    #[inline]
    pub fn swap_chain(&self) -> Arc<Swapchain<Window>> {
        self.swap_chain.clone()
    }

    #[inline]
    pub fn recreate(self) -> Self {
        let (swap_chain, images) = self
            .swap_chain
            .recreate()
            .expect("Failed to recreate swap chain !");

        Self {
            swap_chain,
            images,
            render_pass: Self::create_render_pass(swap_chain.device(), swap_chain.format()),
        }
    }
}
