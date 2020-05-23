use crate::renderer::device_selection::QueueFamilies;
use crate::renderer::swapchain::SwapchainContainer;
use crate::renderer::sync::SyncObjects;
use crate::renderer::ubo::UniformBufferObject;
use crate::renderer::{
    ENGINE_VERSION, MAX_FRAMES_IN_FLIGHT, VULKAN_VERSION, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH,
};
use crate::APPLICATION_VERSION;
#[cfg(feature = "validation-layers")]
use ash::extensions::ext::DebugUtils;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use nalgebra::{Rotation3, Vector3};
use std::collections::HashSet;
use std::ffi::CString;
use winit::event_loop::EventLoop;

pub struct VulkanApp {
    _entry: ash::Entry,
    pub(super) instance: ash::Instance,
    window: winit::window::Window,

    pub(super) surface_container: SurfaceContainer,

    pub(super) physical_device: vk::PhysicalDevice,
    pub(super) device: ash::Device,

    pub(super) queue_families: QueueFamilies,
    graphics_queue: vk::Queue,
    presentation_queue: vk::Queue,

    pub(super) swapchain_container: SwapchainContainer,
    pub(super) image_views: Vec<vk::ImageView>,
    pub(super) framebuffers: Vec<vk::Framebuffer>,

    pub(super) render_pass: vk::RenderPass,
    pub(super) pipeline_layout: vk::PipelineLayout,
    pub(super) graphics_pipeline: vk::Pipeline,

    pub(super) command_pool: vk::CommandPool,
    pub(super) command_buffers: Vec<vk::CommandBuffer>,

    pub(super) vertex_buffer: vk::Buffer,
    pub(super) vertex_buffer_memory: vk::DeviceMemory,
    pub(super) index_buffer: vk::Buffer,
    pub(super) index_buffer_memory: vk::DeviceMemory,

    ubo: UniformBufferObject,
    pub(super) ubo_layout: vk::DescriptorSetLayout,
    pub(super) uniform_buffers: Vec<vk::Buffer>,
    pub(super) uniform_buffers_memory: Vec<vk::DeviceMemory>,

    descriptor_pool: vk::DescriptorPool,
    pub(super) descriptor_sets: Vec<vk::DescriptorSet>,

    pub(super) sync_objects: SyncObjects,
    current_frame: usize,

    #[cfg(feature = "validation-layers")]
    debug_utils_loader: DebugUtils,
    #[cfg(feature = "validation-layers")]
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

pub(super) struct SurfaceContainer {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
}

// Setup methods
impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let entry = ash::Entry::new().expect("Failed to acquire Vulkan entry point !");
        let window = Self::create_window(event_loop);
        let instance = Self::create_instance(&entry, &window);

        let surface_container = Self::create_surface(&entry, &instance, &window);

        let physical_device = Self::pick_physical_device(&instance, &surface_container);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let (device, queue_families) =
            Self::create_logical_device(&instance, physical_device, &surface_container);

        #[cfg(feature = "validation-layers")]
        let (debug_utils_loader, debug_utils_messenger) =
            Self::setup_debug_utils(&entry, &instance);

        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics, 0) };
        let presentation_queue = unsafe { device.get_device_queue(queue_families.presentation, 0) };

        let swapchain_container = Self::create_swapchain(
            &instance,
            &device,
            physical_device,
            &surface_container,
            &queue_families,
        );

        let image_views = Self::create_image_views(
            &device,
            swapchain_container.format,
            &swapchain_container.images,
        );

        let render_pass = Self::create_render_pass(&device, swapchain_container.format);
        let ubo_layout = Self::create_description_set_layout(&device);
        let (graphics_pipeline, pipeline_layout) = Self::create_graphics_pipeline(
            &device,
            render_pass,
            swapchain_container.extent,
            ubo_layout,
        );

        let framebuffers = Self::create_framebuffers(
            &device,
            render_pass,
            &image_views,
            swapchain_container.extent,
        );

        let command_pool = Self::create_command_pool(&device, &queue_families);

        let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_buffer(
            &instance,
            &device,
            physical_device,
            command_pool,
            graphics_queue,
        );

        let (index_buffer, index_buffer_memory) = Self::create_index_buffer(
            &instance,
            &device,
            physical_device,
            command_pool,
            graphics_queue,
        );

        let (uniform_buffers, uniform_buffers_memory) = Self::create_uniform_buffers(
            &device,
            &physical_device_memory_properties,
            swapchain_container.images.len(),
        );

        let descriptor_pool =
            Self::create_descriptor_pool(&device, swapchain_container.images.len());
        let descriptor_sets = Self::create_descriptor_sets(
            &device,
            descriptor_pool,
            ubo_layout,
            &uniform_buffers,
            swapchain_container.images.len(),
        );

        let ubo = Self::create_ubo(swapchain_container.extent);

        let command_buffers = Self::create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &framebuffers,
            render_pass,
            swapchain_container.extent,
            vertex_buffer,
            index_buffer,
            pipeline_layout,
            &descriptor_sets,
        );

        let sync_objects = Self::create_sync_objects(&device);

        Self {
            _entry: entry,
            window,
            instance,

            surface_container,

            physical_device,
            device,

            queue_families,
            graphics_queue,
            presentation_queue,

            swapchain_container,
            image_views,
            framebuffers,

            render_pass,
            pipeline_layout,
            graphics_pipeline,

            command_pool,
            command_buffers,

            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,

            ubo,
            ubo_layout,
            uniform_buffers,
            uniform_buffers_memory,

            descriptor_pool,
            descriptor_sets,

            sync_objects,
            current_frame: 0,

            #[cfg(feature = "validation-layers")]
            debug_utils_loader,
            #[cfg(feature = "validation-layers")]
            debug_utils_messenger,
        }
    }

    /// Create a [`winit::window::Window`].
    fn create_window(event_loop: &EventLoop<()>) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window")
    }

    /// Create a Vulkan instance.
    fn create_instance(entry: &ash::Entry, window: &winit::window::Window) -> ash::Instance {
        #[cfg(feature = "validation-layers")]
        {
            if !Self::check_validation_layer_support(entry) {
                panic!("Validation layers requested, but not available !");
            }
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("AL Engine").unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .application_version(APPLICATION_VERSION)
            .engine_version(ENGINE_VERSION)
            .api_version(VULKAN_VERSION);

        // Platform specific extensions to enable
        #[allow(unused_mut)]
        let mut extension_names = ash_window::enumerate_required_extensions(window)
            .expect("Failed to gather required Vulkan extensions !")
            .into_iter()
            .map(|extension| extension.as_ptr())
            .collect::<Vec<_>>();

        // Add the debug extension if requested
        #[cfg(feature = "validation-layers")]
        extension_names.push(DebugUtils::name().as_ptr());

        // !!! _required_layers_raw_names contains owned data that need to stay in scope until the instance is created !
        #[cfg(feature = "validation-layers")]
        let (_required_layers_raw_names, required_layers_names) =
            Self::get_validation_layers_raw_owned();

        #[allow(unused_mut)]
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        #[cfg(feature = "validation-layers")]
        let mut messenger_create_info = Self::get_messenger_create_info();
        #[cfg(feature = "validation-layers")]
        let create_info = create_info.push_next(&mut messenger_create_info);

        #[cfg(feature = "validation-layers")]
        let create_info = create_info.enabled_layer_names(&required_layers_names);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance !")
        };

        instance
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> SurfaceContainer {
        let surface = unsafe { ash_window::create_surface(entry, instance, window, None) }
            .expect("Failed to create surface !");

        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        SurfaceContainer {
            surface_loader,
            surface,
        }
    }

    /// Create the logical device and queues from a physical device.
    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &SurfaceContainer,
    ) -> (ash::Device, QueueFamilies) {
        // We can unwrap safely
        let indices = Self::find_queue_families(instance, physical_device, surface).unwrap();

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics);
        unique_queue_families.insert(indices.presentation);

        let queue_priorities = [1.0f32];
        let mut queue_create_infos = Vec::new();
        for family in unique_queue_families {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(family)
                    .queue_priorities(&queue_priorities)
                    .build(),
            )
        }

        // TODO: Add features/extensions here
        let features_to_enable = vk::PhysicalDeviceFeatures::builder().build();
        let enable_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];

        #[cfg(feature = "validation-layers")]
        let (_required_layers_raw_names, required_layers_names) =
            Self::get_validation_layers_raw_owned();

        let device_create_info = {
            let builder = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_features(&features_to_enable)
                .enabled_extension_names(&enable_extensions);

            #[cfg(feature = "validation-layers")]
            let builder = builder.enabled_layer_names(&required_layers_names);

            builder.build()
        };

        // Create the device
        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical device !")
        };

        (device, indices)
    }

    fn update_uniform_buffer(&mut self, current_image: usize, delta_time: f32) {
        use nalgebra::RealField;
        self.ubo.model = Rotation3::from_axis_angle(&Vector3::z_axis(), f32::frac_pi_2() * delta_time).to_homogeneous() * &self.ubo.model;

        let ubos = [&self.ubo];

        let buffer_size =
            (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as vk::DeviceSize;

        unsafe {
            let data_ptr =
                self.device
                    .map_memory(
                        self.uniform_buffers_memory[current_image],
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(*ubos.as_ptr(), ubos.len());

            self.device
                .unmap_memory(self.uniform_buffers_memory[current_image]);
        }
    }
}

// Drawing methods
impl VulkanApp {
    pub fn draw_frame(&mut self, delta_time: f32) {
        let wait_fences = [self.sync_objects.inflight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fences !");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain_container.loader.acquire_next_image(
                self.swapchain_container.swapchain,
                std::u64::MAX,
                self.sync_objects.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );

            match result {
                Ok(image_index) => image_index,
                Err(result) => match result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => panic!("Failed to Acquire Next Image !"),
                },
            }
        };

        self.update_uniform_buffer(image_index as usize, delta_time);

        let wait_semaphores = [self.sync_objects.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync_objects.render_finished_semaphores[self.current_frame]];

        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[self.command_buffers[image_index as usize]])
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fences !");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_info,
                    self.sync_objects.inflight_fences[self.current_frame],
                )
                .expect("Failed to execute Queue Submit !");
        }

        let presentation_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&[self.swapchain_container.swapchain])
            .image_indices(&[image_index])
            .build();

        let result = unsafe {
            self.swapchain_container
                .loader
                .queue_present(self.presentation_queue, &presentation_info)
        };

        let need_new_swapchain = match result {
            Ok(_) => false,
            Err(result) => match result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute Queue Present !"),
            },
        };

        if need_new_swapchain {
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

// Accessors/Mutators
impl VulkanApp {
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            // Wait for frames to finish rendering before destroying stuff
            self.device.device_wait_idle().expect("Failed to wait idle");

            for ((&image_available_semaphore, &render_finished_semaphore), &inflight_fence) in self
                .sync_objects
                .image_available_semaphores
                .iter()
                .zip(self.sync_objects.render_finished_semaphores.iter())
                .zip(self.sync_objects.inflight_fences.iter())
            {
                self.device
                    .destroy_semaphore(image_available_semaphore, None);
                self.device
                    .destroy_semaphore(render_finished_semaphore, None);
                self.device.destroy_fence(inflight_fence, None);
            }

            self.cleanup_swapchain();

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.ubo_layout, None);
            self.uniform_buffers
                .iter()
                .zip(self.uniform_buffers_memory.iter())
                .for_each(|(&uniform_buffer, &uniform_buffer_memory)| {
                    self.device.destroy_buffer(uniform_buffer, None);
                    self.device.free_memory(uniform_buffer_memory, None);
                });

            // After the swapchain destruction because we used this buffer in a draw command.
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);

            self.device.destroy_command_pool(self.command_pool, None);

            self.device.destroy_device(None);

            self.surface_container
                .surface_loader
                .destroy_surface(self.surface_container.surface, None);

            #[cfg(feature = "validation-layers")]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

            self.instance.destroy_instance(None);
        }
    }
}
