//! Vulkan setup.

use std::sync::Arc;

use failure::{format_err, Fallible, ResultExt};
use log::{debug, info};
use vulkano::{
    device::{Device, DeviceExtensions, Queue},
    image::SwapchainImage,
    instance::{Instance, PhysicalDevice},
    swapchain::{PresentMode, Surface, SurfaceTransform, Swapchain},
};
use vulkano_win::{self, VkSurfaceBuild};
use winit::{EventsLoop, Window, WindowBuilder};

/// Initialize vulkan.
#[allow(clippy::type_complexity)]
pub fn setup() -> Fallible<(Arc<Device>, Arc<Queue>, Arc<Surface<Window>>, EventsLoop)> {
    // Create an instance of vulkan.
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None)
            .with_context(|e| format_err!("Failed to create vulkan instance: {}", e))?
    };
    debug!("Successfully created vulkan instance: {:?}", instance);

    // List physical devices.
    for device in PhysicalDevice::enumerate(&instance) {
        debug!(
            "Physical device available [{}]: name={:?}, type={:?}, api_version={:?}",
            device.index(),
            device.name(),
            device.ty(),
            device.api_version()
        );
    }

    // Prepare a window.
    let events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .with_context(|e| format_err!("Failed to create window surface: {}", e))?;

    // Select a physical device.
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .ok_or_else(|| format_err!("No physical devices available"))?;
    info!(
        "Selected physical device: index={:?}, name={:?}, type={:?}, api_version={:?}",
        physical.index(),
        physical.name(),
        physical.ty(),
        physical.api_version()
    );

    // List device queue families.
    for family in physical.queue_families() {
        debug!(
            "Queue family found: id={:?}, count={:?}, \
             graphics={:?}, compute={:?}, transfers={:?}",
            family.id(),
            family.queues_count(),
            family.supports_graphics(),
            family.supports_compute(),
            family.supports_transfers()
        );
    }

    // Select a queue family.
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .ok_or_else(|| format_err!("No graphical queues available"))?;
    info!(
        "Using queue family: id={:?}, count={:?}",
        queue_family.id(),
        queue_family.queues_count()
    );

    // Initialize device.
    let (device, queue) = {
        /// Queue priority, between 0.0 and 1.0.
        ///
        /// This can be any value in the range, because in this program only one
        /// queue family is used.
        const QUEUE_PRIORITY: f32 = 0.5;
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue_family, QUEUE_PRIORITY)].iter().cloned(),
        )
        .with_context(|e| format_err!("Failed to create device: {}", e))?;
        (device, queues.next().expect("Should never fail"))
    };
    info!("Successfully created device object");

    Ok((device, queue, surface, events_loop))
}

/// Create swapchain.
#[allow(clippy::type_complexity)]
pub fn create_swapchain(
    device: &Arc<Device>,
    queue: &Arc<Queue>,
    surface: &Arc<Surface<Window>>,
) -> Fallible<(Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>)> {
    let caps = surface
        .capabilities(device.physical_device())
        .with_context(|e| format_err!("Failed to get surface capabilities: {}", e))?;
    debug!("Capabilities: {:?}", caps);
    let usage = caps.supported_usage_flags;
    let alpha = caps
        .supported_composite_alpha
        .iter()
        .next()
        .ok_or_else(|| format_err!("No desired composite alpha modes are supported"))?;
    info!("Selected alpha composite mode: {:?}", alpha);
    let format = caps.supported_formats[0].0;
    info!("Selected swapchain format: {:?}", format);

    let window = surface.window();
    let initial_dimensions = window
        .get_inner_size()
        .map(|dimensions| {
            // Convert to physical pixels
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
        })
        .ok_or_else(|| format_err!("The window no longer exists"))?;
    let (swapchain, image) = Swapchain::new(
        device.clone(),
        surface.clone(),
        caps.min_image_count,
        format,
        initial_dimensions,
        1,
        usage,
        queue,
        SurfaceTransform::Identity,
        alpha,
        PresentMode::Fifo,
        true,
        None,
    )
    .with_context(|e| format_err!("Failed to create swapchain: {}", e))?;
    Ok((swapchain, image))
}
