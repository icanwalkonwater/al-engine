//! This module extends [`VulkanApp`] to implement the swapchain creation.

use crate::renderer::device_selection::QueueFamilies;
use crate::renderer::vulkan_app::{SurfaceContainer, VulkanApp};
use crate::renderer::{WINDOW_HEIGHT, WINDOW_WIDTH};
use ash::version::DeviceV1_0;
use ash::vk;
use ash::vk::Extent2D;
use log::warn;

pub(in crate::renderer) struct SwapchainContainer {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

pub(in crate::renderer) struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub presentation_modes: Vec<vk::PresentModeKHR>,
}

impl VulkanApp {
    /// Create the swapchain and its images.
    pub(super) fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface_container: &SurfaceContainer,
        queue_families: &QueueFamilies,
    ) -> SwapchainContainer {
        let support = Self::query_swapchain_support(physical_device, surface_container);

        let format = Self::choose_swapchain_format(&support.formats);
        let presentation_mode =
            Self::choose_swapchain_presentation_mode(&support.presentation_modes);
        let extent = Self::choose_swapchain_extent(&support.capabilities);

        // Recommended: min + 1.
        // But must be in bounds with the max.
        // Note: max <= 0 means no maximum.
        let image_count = support.capabilities.min_image_count + 1;
        let image_count = if support.capabilities.max_image_count > 0 {
            image_count.min(support.capabilities.max_image_count)
        } else {
            image_count
        };

        // Swapchain sharing info
        let (image_sharing_mode, family_indices) =
            if queue_families.graphics != queue_families.presentation {
                (
                    vk::SharingMode::EXCLUSIVE,
                    vec![queue_families.graphics, queue_families.presentation],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, Vec::new())
            };

        // Swapchain/loader create
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface_container.surface)
            .min_image_count(image_count)
            .image_color_space(format.color_space)
            .image_format(format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&family_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(presentation_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create swapchain !")
        };

        // Swapchain images create
        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to query swapchain images")
        };

        SwapchainContainer {
            loader: swapchain_loader,
            swapchain,
            images: swapchain_images,
            format: format.format,
            extent: extent,
        }
    }

    /// Recreate (in this order)
    /// - The swapchain
    /// - The image views
    /// - The render pass
    /// - The graphics pipeline & layout
    /// - The framebuffers
    /// - The command buffers
    pub(super) fn recreate_swapchain(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait until device is idle !");
        }

        self.cleanup_swapchain();

        let swapchain_container = Self::create_swapchain(
            &self.instance,
            &self.device,
            self.physical_device,
            &self.surface_container,
            &self.queue_families,
        );

        self.swapchain_container = swapchain_container;

        self.image_views = Self::create_image_views(
            &self.device,
            self.swapchain_container.format,
            &self.swapchain_container.images,
        );

        self.render_pass = Self::create_render_pass(&self.device, self.swapchain_container.format);

        let (graphics_pipeline, pipeline_layout) = Self::create_graphics_pipeline(
            &self.device,
            self.render_pass,
            self.swapchain_container.extent,
            self.ubo_layout,
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        self.framebuffers = Self::create_framebuffers(
            &self.device,
            self.render_pass,
            &self.image_views,
            self.swapchain_container.extent,
        );

        self.command_buffers = Self::create_command_buffers(
            &self.device,
            self.command_pool,
            self.graphics_pipeline,
            &self.framebuffers,
            self.render_pass,
            self.swapchain_container.extent,
            self.vertex_buffer,
            self.index_buffer,
            self.pipeline_layout,
            &self.descriptor_sets,
        );

        self.ubo = Self::create_ubo(self.swapchain_container.extent);
    }

    /// Destroys command buffers, graphics pipeline, pipeline layout, render pass, image views and swapchain.
    pub(super) fn cleanup_swapchain(&self) {
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &image_view in self.image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain_container
                .loader
                .destroy_swapchain(self.swapchain_container.swapchain, None);
        }
    }

    pub(super) fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface_container: &SurfaceContainer,
    ) -> SwapchainSupport {
        unsafe {
            let capabilities = surface_container
                .surface_loader
                .get_physical_device_surface_capabilities(
                    physical_device,
                    surface_container.surface,
                )
                .expect("Failed to query surface capabilities !");

            let formats = surface_container
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface_container.surface)
                .expect("Failed to query surface formats !");

            let presentation_modes = surface_container
                .surface_loader
                .get_physical_device_surface_present_modes(
                    physical_device,
                    surface_container.surface,
                )
                .expect("Failed to query surface presentation modes !");

            SwapchainSupport {
                capabilities,
                formats,
                presentation_modes,
            }
        }
    }

    /// Create image views to access swapchain images.
    pub(super) fn create_image_views(
        device: &ash::Device,
        surface_format: vk::Format,
        images: &[vk::Image],
    ) -> Vec<vk::ImageView> {
        images
            .iter()
            .map(|&image| {
                let image_view_create_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format)
                    .components(
                        vk::ComponentMapping::builder()
                            .r(vk::ComponentSwizzle::IDENTITY)
                            .g(vk::ComponentSwizzle::IDENTITY)
                            .b(vk::ComponentSwizzle::IDENTITY)
                            .a(vk::ComponentSwizzle::IDENTITY)
                            .build(),
                    )
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    )
                    .image(image);

                unsafe {
                    device
                        .create_image_view(&image_view_create_info, None)
                        .expect("Failed to create image view !")
                }
            })
            .collect()
    }

    /// Preferred format is B8G8R8A8_UNORM.
    /// Preferred color space is SRGB_NONLINEAR.
    /// Falls back to whatever comes first.
    fn choose_swapchain_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        let format = formats.iter().find(|format| {
            format.format == vk::Format::B8G8R8A8_UNORM
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        });

        if let Some(&format) = format {
            format
        } else {
            let format = formats[0];
            warn!(
                "Ideal surface format not found, falling back to {:?}/{:?}",
                format.format, format.color_space
            );
            format
        }
    }

    // Prefer MAILBOX mode, falls back to IMMEDIATE and then to the default FIFO.
    fn choose_swapchain_presentation_mode(modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        let mailbox = modes
            .iter()
            .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX);

        if let Some(_) = mailbox {
            vk::PresentModeKHR::MAILBOX
        } else {
            warn!("Mailbox presentation mode not found, falling back to Immediate");
            let immediate = modes
                .iter()
                .find(|&&mode| mode == vk::PresentModeKHR::IMMEDIATE);

            if let Some(_) = immediate {
                vk::PresentModeKHR::IMMEDIATE
            } else {
                warn!("Immediate presentation mode not found, falling back to FIFO");
                vk::PresentModeKHR::FIFO
            }
        }
    }

    /// The extent size is the window size clamped in the range supported by the surface.
    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: nalgebra::clamp(
                    WINDOW_WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: nalgebra::clamp(
                    WINDOW_HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}
