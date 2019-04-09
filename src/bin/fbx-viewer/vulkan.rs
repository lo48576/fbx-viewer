//! Vulkan version.

use std::sync::Arc;

use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};
use failure::{bail, format_err, Fallible, ResultExt};
use fbx_viewer::{fbx, CliOpt};
use log::{error, info, trace};
use vulkano::{
    buffer::{BufferUsage, CpuBufferPool},
    command_buffer::AutoCommandBufferBuilder,
    descriptor::descriptor_set::PersistentDescriptorSet,
    device::Device,
    format::Format,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{AttachmentImage, SwapchainImage},
    pipeline::{
        vertex::SingleBufferDefinition, viewport::Viewport, GraphicsPipeline,
        GraphicsPipelineAbstract,
    },
    swapchain::{AcquireError, SwapchainCreationError},
    sync::GpuFuture,
};
use winit::Window;

use self::setup::{create_swapchain, setup};

mod setup;

/// Depth format.
const DEPTH_FORMAT: Format = Format::D32Sfloat;

pub fn main(opt: CliOpt) -> Fallible<()> {
    info!("Vulkan mode");

    let (device, queue, surface, mut events_loop) =
        setup().with_context(|e| format_err!("Failed to setup vulkan: {}", e))?;
    let window = surface.window();
    let mut dimensions = window_dimensions(&window)
        .with_context(|e| format_err!("Failed to get window dimensions: {}", e))?;
    let (mut swapchain, images) = create_swapchain(&device, &queue, &surface)
        .with_context(|e| format_err!("Failed to create swapchain: {}", e))?;

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

    let vs = vs::Shader::load(device.clone())
        .with_context(|e| format_err!("Failed to load vertex shader: {}", e))?;
    let fs = fs::Shader::load(device.clone())
        .with_context(|e| format_err!("Failed to load fragment shader: {}", e))?;

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: DEPTH_FORMAT,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .with_context(|e| format_err!("Failed to create render pass: {}", e))?,
    );

    let (mut pipeline, mut framebuffers) =
        window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone())
            .with_context(|e| format_err!("Failed to set up pipeline and framebuffers: {}", e))?;
    let mut recreate_swapchain = false;

    let _scene = fbx::load(&opt.fbx_path)
        .with_context(|e| format_err!("Failed to interpret FBX scene: {}", e))?;

    let mut previous_frame: Box<dyn GpuFuture> = Box::new(vulkano::sync::now(device.clone()));
    let rotation_start = std::time::Instant::now();

    previous_frame
        .flush()
        .with_context(|e| format_err!("Failed to prepare resources: {}", e))?;
    'main: loop {
        previous_frame.cleanup_finished();
        if recreate_swapchain {
            trace!("Recreating swapchain");
            dimensions = window_dimensions(&window)
                .with_context(|e| format_err!("Failed to get window dimensions: {}", e))?;

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(e) => bail!("Failed to recreate swapchain: {}", e),
            };
            swapchain = new_swapchain;

            let (new_pipeline, new_framebuffers) = window_size_dependent_setup(
                device.clone(),
                &vs,
                &fs,
                &new_images,
                render_pass.clone(),
            )
            .with_context(|e| format_err!("Failed to set up pipeline and framebuffers: {}", e))?;
            pipeline = new_pipeline;
            framebuffers = new_framebuffers;

            trace!("Swapchain recreation done");
            recreate_swapchain = false;
        }

        let uniform_buffer_subbuffer = {
            let elapsed = rotation_start.elapsed();
            let rotation =
                elapsed.as_secs() as f64 + f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0;
            let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

            let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
            let proj =
                cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
            let eye = Point3::new(0.0, 0.5, -1.5);
            let center = Point3::new(0.0, 1.0, 0.0);
            let up = Vector3::new(0.0, -1.0, 0.0);
            let view = Matrix4::look_at(eye, center, up);
            let scale = Matrix4::from_scale(0.01);

            let uniform_data = vs::ty::Data {
                world: Matrix4::from(rotation).into(),
                view: (view * scale).into(),
                proj: proj.into(),
            };

            uniform_buffer
                .next(uniform_data)
                .with_context(|e| format_err!("Failed to put data into uniform buffer: {}", e))?
        };

        let set0 = Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 0)
                .add_buffer(uniform_buffer_subbuffer)
                .with_context(|e| {
                    format_err!("Failed to add uniform buffer to descriptor set: {}", e)
                })?
                .build()
                .with_context(|e| format_err!("Failed to build descriptor set: {}", e))?,
        );
        let (image_num, acquire_future) =
            match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                }
                Err(e) => bail!("`acquire_next_image()` failed: {}", e),
            };

        let command_buffer =
            {
                let builder = AutoCommandBufferBuilder::primary_one_time_submit(
                    device.clone(),
                    queue.family(),
                )
                .with_context(|e| format_err!("Failed to create command buffer builder: {}", e))?
                .begin_render_pass(
                    framebuffers[image_num].clone(),
                    false,
                    vec![[0.0, 0.0, 1.0, 1.0].into(), 1f32.into()],
                )
                .with_context(|e| format_err!("Failed to begin new render pass creation: {}", e))?;
                // TODO: Draw scene here.
                let _ = set0.clone();
                builder
                    .end_render_pass()
                    .with_context(|e| format_err!("Failed to end a render pass creation: {}", e))?
                    .build()
                    .with_context(|e| format_err!("Failed to build a new command buffer: {}", e))?
            };

        let future = previous_frame
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .with_context(|e| format_err!("Failed to execute command buffer: {}", e))?
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();
        match future {
            Ok(future) => {
                previous_frame = Box::new(future) as Box<_>;
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
            Err(e) => {
                error!("{}", e);
                previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;
        events_loop.poll_events(|ev| {
            use winit::{Event, WindowEvent};
            match ev {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => done = true,
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => recreate_swapchain = true,
                _ => {}
            }
        });
        if done {
            break 'main;
        }
    }

    Ok(())
}

/// Setups pipeline and framebuffers.
#[allow(clippy::type_complexity)]
fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
) -> Fallible<(
    Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
)> {
    let dimensions = images[0].dimensions();
    let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, DEPTH_FORMAT)
        .with_context(|e| format_err!("Failed to create depth buffer: {}", e))?;

    let framebuffers = images
        .iter()
        .map(|image| {
            Framebuffer::start(render_pass.clone())
                .add(image.clone())
                .with_context(|e| {
                    format_err!("Failed to add a swapchain image to framebuffer: {}", e)
                })?
                .add(depth_buffer.clone())
                .with_context(|e| {
                    format_err!("Failed to add a depth buffer to framebuffer: {}", e)
                })?
                .build()
                .map(|fb| Arc::new(fb) as Arc<dyn FramebufferAbstract + Send + Sync>)
                .with_context(|e| format_err!("Failed to create framebuffer: {}", e))
                .map_err(Into::into)
        })
        .collect::<Fallible<Vec<_>>>()
        .with_context(|e| format_err!("Failed to create framebuffers: {}", e))?;

    let pipeline = GraphicsPipeline::start()
        .vertex_input(SingleBufferDefinition::<fbx_viewer::drawable::vulkan::Vertex>::new())
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .viewports(std::iter::once(Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        }))
        .fragment_shader(fs.main_entry_point(), ())
        .blend_alpha_blending()
        .depth_stencil_simple_depth()
        .render_pass(
            Subpass::from(render_pass.clone(), 0)
                .ok_or_else(|| format_err!("Failed to create subpass"))?,
        )
        .build(device.clone())
        .map(Arc::new)
        .with_context(|e| format_err!("Failed to create pipeline: {}", e))?;

    Ok((pipeline, framebuffers))
}

/// Returns window dimensions.
fn window_dimensions(window: &Window) -> Fallible<[u32; 2]> {
    window
        .get_inner_size()
        .map(|dimensions| {
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
        })
        .ok_or_else(|| format_err!("Window no longer exists"))
        .map_err(Into::into)
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/bin/fbx-viewer/shaders/default.vert",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/bin/fbx-viewer/shaders/default.frag",
    }
}
