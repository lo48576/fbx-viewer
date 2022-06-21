//! Vulkan setup.

use std::sync::Arc;

use anyhow::{anyhow, Context};
use log::{debug, info};
use vulkano::{
    descriptor::{
        descriptor_set::{DescriptorSet, PersistentDescriptorSet},
        pipeline_layout::PipelineLayoutAbstract,
    },
    device::{Device, DeviceExtensions, Queue},
    format::R8G8B8A8Srgb,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    pipeline::GraphicsPipeline,
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
    swapchain::{
        ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform, Swapchain,
    },
    sync::GpuFuture,
};
use vulkano_win::{self, VkSurfaceBuild};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

/// Initialize vulkan.
#[allow(clippy::type_complexity)]
pub fn setup() -> anyhow::Result<(Arc<Device>, Arc<Queue>, Arc<Surface<Window>>, EventLoop<()>)> {
    // Create an instance of vulkan.
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).context("Failed to create vulkan instance")?
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
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .context("Failed to create window surface")?;

    // Select a physical device.
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .ok_or_else(|| anyhow!("No physical devices available"))?;
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
            "Queue family found: id={:?}, count={:?}, graphics={:?}, compute={:?}",
            family.id(),
            family.queues_count(),
            family.supports_graphics(),
            family.supports_compute(),
        );
    }

    // Select a queue family.
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .ok_or_else(|| anyhow!("No graphical queues available"))?;
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
        .context("Failed to create device")?;
        (device, queues.next().expect("Should never fail"))
    };
    info!("Successfully created device object");

    Ok((device, queue, surface, event_loop))
}

/// Create swapchain.
#[allow(clippy::type_complexity)]
pub fn create_swapchain(
    device: &Arc<Device>,
    queue: &Arc<Queue>,
    surface: &Arc<Surface<Window>>,
) -> anyhow::Result<(Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>)> {
    let caps = surface
        .capabilities(device.physical_device())
        .context("Failed to get surface capabilities")?;
    debug!("Capabilities: {:?}", caps);
    let usage = caps.supported_usage_flags;
    let alpha = caps
        .supported_composite_alpha
        .iter()
        .next()
        .ok_or_else(|| anyhow!("No desired composite alpha modes are supported"))?;
    info!("Selected alpha composite mode: {:?}", alpha);
    let format = caps.supported_formats[0].0;
    info!("Selected swapchain format: {:?}", format);

    let window = surface.window();
    let (swapchain, image) = Swapchain::new(
        device.clone(),
        surface.clone(),
        caps.min_image_count,
        format,
        window.inner_size().into(),
        1,
        usage,
        queue,
        SurfaceTransform::Identity,
        alpha,
        PresentMode::Fifo,
        FullscreenExclusive::Default,
        true,
        ColorSpace::SrgbNonLinear,
    )
    .context("Failed to create swapchain")?;
    Ok((swapchain, image))
}

/// Creates dummy 1x1 white texture.
#[allow(clippy::type_complexity)]
pub fn create_dummy_texture(
    device: Arc<Device>,
    queue: Arc<Queue>,
) -> anyhow::Result<(
    Arc<ImmutableImage<R8G8B8A8Srgb>>,
    Arc<Sampler>,
    Box<dyn GpuFuture>,
)> {
    let raw_image = [0xffu8; 4];
    let dim = ImageDimensions::Dim2d {
        width: 1,
        height: 1,
        array_layers: 1,
    };
    let (image, img_future) = ImmutableImage::from_iter(
        raw_image.iter().cloned(),
        dim,
        MipmapsCount::One,
        R8G8B8A8Srgb,
        queue,
    )
    .context("Failed to upload dummy texture image")?;
    let sampler = Sampler::new(
        device,
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0,
        1.0,
        0.0,
        0.0,
    )
    .context("Failed to create sampler")?;

    Ok((image, sampler, Box::new(img_future)))
}

/// Creates a descriptor set for the given diffuse texture.
pub fn create_diffuse_texture_desc_set<Mv, L, Rp>(
    image: Arc<ImageView<Arc<ImmutableImage<R8G8B8A8Srgb>>>>,
    sampler: Arc<Sampler>,
    pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
) -> anyhow::Result<Arc<dyn DescriptorSet + Send + Sync>>
where
    L: PipelineLayoutAbstract,
{
    let layout = pipeline
        .layout()
        .descriptor_set_layout(1)
        .context("Failed to get the second descriptor set layout of the pipeline")?;
    let desc_set = PersistentDescriptorSet::start(layout.clone())
        .add_sampled_image(image, sampler)
        .context("Failed to add sampled image to descriptor set")?
        .build()
        .context("Failed to build descriptor set")?;

    Ok(Arc::new(desc_set) as Arc<_>)
}
