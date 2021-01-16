//! Vulkan version.

use std::sync::Arc;

use anyhow::{anyhow, Context};
use cgmath::{
    Angle, EuclideanSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3,
};
use fbx_viewer::{fbx, CliOpt};
use log::{debug, error, info, trace};
use vulkano::{
    buffer::{BufferUsage, CpuBufferPool},
    command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents},
    descriptor::{
        descriptor_set::{DescriptorSet, PersistentDescriptorSet},
        pipeline_layout::PipelineLayoutAbstract,
    },
    device::Device,
    format::Format,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{AttachmentImage, SwapchainImage},
    pipeline::{vertex::SingleBufferDefinition, viewport::Viewport, GraphicsPipeline},
    swapchain::{AcquireError, SwapchainCreationError},
    sync::GpuFuture,
};
use winit::window::Window;

use self::setup::{create_diffuse_texture_desc_set, create_dummy_texture, create_swapchain, setup};

mod drawable;
mod setup;

/// Depth format.
const DEPTH_FORMAT: Format = Format::D32Sfloat;

pub fn main(opt: CliOpt) -> anyhow::Result<()> {
    info!("Vulkan mode");

    let (device, queue, surface, event_loop) = setup().context("Failed to setup vulkan")?;
    let window = surface.window();
    let mut dimensions = window.inner_size().into();
    let (mut swapchain, images) =
        create_swapchain(&device, &queue, &surface).context("Failed to create swapchain")?;

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

    let vs = vs::Shader::load(device.clone()).context("Failed to load vertex shader")?;
    let fs = fs::Shader::load(device.clone()).context("Failed to load fragment shader")?;

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
        .context("Failed to create render pass")?,
    );

    let (mut pipeline, mut framebuffers) =
        window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone())
            .context("Failed to set up pipeline and framebuffers")?;
    let mut recreate_swapchain = false;

    let mut previous_frame: Box<dyn GpuFuture> = vulkano::sync::now(device.clone()).boxed();

    let (dummy_texture_image, dummy_texture_sampler, dummy_texture_future) =
        create_dummy_texture(device.clone(), queue.clone())
            .context("Failed to create dummy texture")?;
    previous_frame = previous_frame.join(dummy_texture_future).boxed();

    let scene = fbx::load(&opt.fbx_path).context("Failed to interpret FBX scene")?;
    let (mut drawable_scene, drawable_scene_future) =
        drawable::Loader::new(device.clone(), queue.clone())
            .load(&scene)
            .context("Failed to load scene as drawable data")?;
    drop(scene);
    let scene_bbox = drawable_scene
        .bbox()
        .bounding_box()
        .ok_or_else(|| anyhow!("No data to show (bounding box is `None`)"))?;
    info!("Scene bounding box = {:?}", scene_bbox);
    if let Some(future) = drawable_scene_future {
        previous_frame = previous_frame.join(future).boxed();
    }
    previous_frame = drawable_scene
        .reset_cache_with_pipeline(&pipeline)?
        .unwrap_or_else(|| vulkano::sync::now(device.clone()).boxed())
        .join(previous_frame)
        .boxed();
    let mut dummy_texture_desc_set = create_diffuse_texture_desc_set(
        dummy_texture_image.clone(),
        dummy_texture_sampler.clone(),
        pipeline.clone(),
    )?;

    let initial_camera = {
        let center = Point3::midpoint(scene_bbox.min(), scene_bbox.max()).map(Into::into);
        debug!("Center calculated from the bounding box: {:?}", center);
        let size: Vector3<f64> = scene_bbox.size().map(Into::into);
        let distance = size[0].max(size[1]);
        let position = Point3::new(center.x, center.y, center.z + distance);
        Camera::with_position(position)
    };
    debug!("Initial camera = {:?}", initial_camera);
    let mut camera = initial_camera;

    previous_frame
        .flush()
        .context("Failed to prepare resources")?;

    let mut kbd_modifiers = winit::event::ModifiersState::default();

    // Use `Option<_>`, since `GpuFuture::then_signal_fence_and_flush()` takes the ownership of the
    // future (`self`) and `previous_frame` would be temporarily empty.
    let mut previous_frame: Option<Box<dyn GpuFuture>> = Some(previous_frame);
    event_loop.run(move |event, _target_window, cflow| {
        use winit::{
            event::{DeviceEvent, ElementState, Event, KeyboardInput, ScanCode, WindowEvent},
            event_loop::ControlFlow,
        };

        let window = surface.window();

        match event {
            Event::RedrawEventsCleared => {
                previous_frame
                    .as_mut()
                    .expect(
                        "Should never fail: a future for the previous frame should be available",
                    )
                    .cleanup_finished();

                if recreate_swapchain {
                    trace!("Recreating swapchain");
                    dimensions = window.inner_size().into();

                    let (new_swapchain, new_images) =
                        match swapchain.recreate_with_dimensions(dimensions) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {}", e),
                        };
                    swapchain = new_swapchain;

                    let (new_pipeline, new_framebuffers) = window_size_dependent_setup(
                        device.clone(),
                        &vs,
                        &fs,
                        &new_images,
                        render_pass.clone(),
                    )
                    .expect("Failed to set up pipeline and framebuffers");
                    pipeline = new_pipeline;
                    framebuffers = new_framebuffers;

                    dummy_texture_desc_set = create_diffuse_texture_desc_set(
                        dummy_texture_image.clone(),
                        dummy_texture_sampler.clone(),
                        pipeline.clone(),
                    )
                    .expect("Failed to create diffuse texture descriptor set");
                    previous_frame = Some(
                        drawable_scene
                            .reset_cache_with_pipeline(&pipeline)
                            .expect("Failed to reset scene cash")
                            .unwrap_or_else(|| vulkano::sync::now(device.clone()).boxed()),
                    );

                    trace!("Swapchain recreation done");
                    recreate_swapchain = false;
                }
                let uniform_buffer_subbuffer = {
                    let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;

                    /// Conversion from GL coordinate system to Vulkan coordinate
                    /// system.
                    ///
                    /// See <https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/>.
                    const PROJ_GL_TO_VULKAN: Matrix4<f32> = Matrix4::new(
                        1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0,
                        1.0,
                    );
                    let proj = PROJ_GL_TO_VULKAN
                        * cgmath::perspective(Rad::turn_div_6(), aspect_ratio, 0.1, 1000.0);
                    let view: Matrix4<f32> = camera
                        .view()
                        .cast()
                        .unwrap_or_else(|| panic!("Abnormal camera posture: {:?}", camera));
                    let world = <Matrix4<f32> as cgmath::SquareMatrix>::identity();
                    let uniform_data = vs::ty::Data {
                        world: world.into(),
                        view: view.into(),
                        proj: proj.into(),
                    };

                    uniform_buffer
                        .next(uniform_data)
                        .expect("Failed to put data into uniform buffer")
                };
                let set0 = {
                    let layout = pipeline
                        .layout()
                        .descriptor_set_layout(0)
                        .expect("Failed to get the first descriptor set layout of the pipeline");
                    Arc::new(
                        PersistentDescriptorSet::start(layout.clone())
                            .add_buffer(uniform_buffer_subbuffer)
                            .expect("Failed to add uniform buffer to descriptor set")
                            .build()
                            .expect("Failed to build descriptor set"),
                    )
                };
                let (image_num, is_suboptimal, acquire_future) =
                    match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("`acquire_next_image()` failed: {}", e),
                    };
                if is_suboptimal {
                    recreate_swapchain = true;
                }

                let command_buffer = {
                    let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
                        device.clone(),
                        queue.family(),
                    )
                    .expect("Failed to create command buffer builder");

                    builder
                        .begin_render_pass(
                            framebuffers[image_num].clone(),
                            SubpassContents::Inline,
                            vec![[0.0, 0.0, 1.0, 1.0].into(), 1f32.into()],
                        )
                        .expect("Failed to begin new render pass creation");

                    // TODO: Draw scene here.
                    let mut opaque_meshes = Vec::new();
                    let mut transparent_meshes = Vec::new();
                    for mesh in &drawable_scene.meshes {
                        let geometry_mesh_i = mesh.geometry_mesh_index;
                        let geometry_mesh = drawable_scene
                            .geometry_mesh(geometry_mesh_i)
                            .unwrap_or_else(|| {
                                panic!("Geometry mesh index out of range: {:?}", geometry_mesh_i)
                            });
                        for (&material_i, index_buffer) in mesh
                            .materials
                            .iter()
                            .zip(geometry_mesh.indices_per_material.iter())
                        {
                            let material =
                                drawable_scene.material(material_i).unwrap_or_else(|| {
                                    panic!("Material index out of range: {:?}", material_i)
                                });
                            let material_desc_set = material
                                .cache
                                .uniform_buffer
                                .as_ref()
                                .expect("Material uniform buffer should be uploaded");
                            let texture = material.diffuse_texture.map(|diffuse_i| {
                                drawable_scene.texture(diffuse_i).unwrap_or_else(|| {
                                    panic!("Material index out of range: {:?}", material_i)
                                })
                            });
                            let texture_desc_set: Arc<dyn DescriptorSet + Send + Sync> = texture
                                .map_or_else(
                                    || dummy_texture_desc_set.clone(),
                                    |t| {
                                        t.cache
                                    .descriptor_set
                                    .as_ref()
                                    .expect(
                                        "Descriptor set for texture should be initialized but not",
                                    )
                                    .clone()
                                    },
                                );
                            let stuff = (
                                geometry_mesh.vertices.clone(),
                                index_buffer.clone(),
                                material_desc_set.clone(),
                                texture_desc_set,
                            );
                            if texture.map_or(false, |t| t.transparent) {
                                transparent_meshes.push(stuff);
                            } else {
                                opaque_meshes.push(stuff);
                            }
                        }
                    }

                    // TODO: Draw the whole scene, not only meshes.
                    for (vertex, index, material, texture_desc_set) in
                        opaque_meshes.into_iter().chain(transparent_meshes)
                    {
                        builder
                            .draw_indexed(
                                pipeline.clone(),
                                &DynamicState::none(),
                                vertex,
                                index,
                                (set0.clone(), texture_desc_set.clone(), material.clone()),
                                (),
                            )
                            .expect("Failed to add a draw call to command buffer");
                    }

                    builder
                        .end_render_pass()
                        .expect("Failed to end a render pass creation");

                    builder
                        .build()
                        .expect("Failed to build a new command buffer")
                };

                let future = previous_frame
                    .take()
                    .expect(
                        "Should never fail: a future for the previous frame should be available",
                    )
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .expect("Failed to execute command buffer")
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();
                match future {
                    Ok(future) => {
                        previous_frame = Some(future.boxed());
                    }
                    Err(vulkano::sync::FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame = Some(vulkano::sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        error!("{}", e);
                        previous_frame = Some(vulkano::sync::now(device.clone()).boxed());
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *cflow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => recreate_swapchain = true,
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(modifiers),
                ..
            } => kbd_modifiers = modifiers,
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Key(input) => {
                    const FORWARD: ScanCode = 17;
                    const BACK: ScanCode = 31;
                    const LEFT: ScanCode = 30;
                    const RIGHT: ScanCode = 32;
                    const ZERO: ScanCode = 11;
                    let move_delta = {
                        let bbox_size = scene_bbox.size();
                        let min_div_32 = bbox_size[0].min(bbox_size[1]).min(bbox_size[2]) / 32.0;
                        let max_div_128 = bbox_size[0].max(bbox_size[1]).max(bbox_size[2]) / 128.0;
                        f64::from(min_div_32.max(max_div_128))
                    };
                    const ANGLE_DELTA: Rad<f64> = Rad(std::f64::consts::FRAC_PI_2 / 16.0);
                    match input {
                        KeyboardInput {
                            scancode: FORWARD,
                            state: ElementState::Pressed,
                            ..
                        } => {
                            if kbd_modifiers.shift() {
                                camera.move_rel(Camera::up() * move_delta);
                            } else if kbd_modifiers.ctrl() {
                                camera.rotate_up(ANGLE_DELTA);
                            } else {
                                camera.move_rel(Camera::forward() * move_delta);
                            }
                        }
                        KeyboardInput {
                            scancode: BACK,
                            state: ElementState::Pressed,
                            ..
                        } => {
                            if kbd_modifiers.shift() {
                                camera.move_rel(Camera::up() * -move_delta);
                            } else if kbd_modifiers.ctrl() {
                                camera.rotate_up(-ANGLE_DELTA);
                            } else {
                                camera.move_rel(Camera::forward() * -move_delta);
                            }
                        }
                        KeyboardInput {
                            scancode: LEFT,
                            state: ElementState::Pressed,
                            ..
                        } => {
                            if kbd_modifiers.ctrl() {
                                camera.rotate_right(-ANGLE_DELTA);
                            } else {
                                camera.move_rel(Camera::right() * -move_delta);
                            }
                        }
                        KeyboardInput {
                            scancode: RIGHT,
                            state: ElementState::Pressed,
                            ..
                        } => {
                            if kbd_modifiers.ctrl() {
                                camera.rotate_right(ANGLE_DELTA);
                            } else {
                                camera.move_rel(Camera::right() * move_delta);
                            }
                        }
                        KeyboardInput {
                            scancode: ZERO,
                            state: ElementState::Pressed,
                            ..
                        } => {
                            if kbd_modifiers.ctrl() {
                                camera.yaw = initial_camera.yaw;
                                camera.pitch = initial_camera.pitch;
                                trace!("Reset camera posture: camera = {:?}", camera);
                            } else {
                                camera.position = initial_camera.position;
                                trace!("Reset camera position: camera = {:?}", camera);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
}

/// Setups pipeline and framebuffers.
#[allow(clippy::type_complexity)]
fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
) -> anyhow::Result<(
    Arc<
        GraphicsPipeline<
            SingleBufferDefinition<drawable::vertex::Vertex>,
            Box<dyn PipelineLayoutAbstract + Send + Sync>,
            Arc<dyn RenderPassAbstract + Send + Sync>,
        >,
    >,
    Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
)> {
    let dimensions = images[0].dimensions();
    let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, DEPTH_FORMAT)
        .context("Failed to create depth buffer")?;

    let framebuffers = images
        .iter()
        .map(|image| {
            Framebuffer::start(render_pass.clone())
                .add(image.clone())
                .context("Failed to add a swapchain image to framebuffer")?
                .add(depth_buffer.clone())
                .context("Failed to add a depth buffer to framebuffer")?
                .build()
                .map(|fb| Arc::new(fb) as Arc<dyn FramebufferAbstract + Send + Sync>)
                .context("Failed to create framebuffer")
                .map_err(Into::into)
        })
        .collect::<anyhow::Result<Vec<_>>>()
        .context("Failed to create framebuffers")?;

    let pipeline = GraphicsPipeline::start()
        .vertex_input(SingleBufferDefinition::<drawable::Vertex>::new())
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
                .ok_or_else(|| anyhow!("Failed to create subpass"))?,
        )
        .build(device)
        .map(Arc::new)
        .context("Failed to create pipeline")?;

    Ok((pipeline, framebuffers))
}

/// Camera.
#[derive(Debug, Copy, Clone, PartialEq)]
struct Camera {
    /// Eye position.
    pub position: Point3<f64>,
    /// Yaw.
    ///
    /// Positive is clockwise.
    pub yaw: Rad<f64>,
    /// Pitch.
    ///
    /// Positive is up.
    pub pitch: Rad<f64>,
    /// Scale.
    pub scale: f64,
}

impl Camera {
    /// Returns the forward direction vector.
    fn forward() -> Vector3<f64> {
        -Vector3::unit_z()
    }

    /// Returns the up direction vector.
    fn up() -> Vector3<f64> {
        Vector3::unit_y()
    }

    /// Returns the right direction vector.
    fn right() -> Vector3<f64> {
        Vector3::unit_x()
    }

    /// Creates a new `Camera` with the given initial position.
    pub fn with_position(position: Point3<f64>) -> Self {
        Self {
            position,
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            scale: 1.0,
        }
    }

    /// Returns view matrix.
    pub fn view(&self) -> Matrix4<f64> {
        Matrix4::from_scale(self.scale)
            * Matrix4::from(self.camera_direction().conjugate())
            * Matrix4::from_translation(-self.position.to_vec())
    }

    /// Returns the direction the camera is looking at.
    fn camera_direction(&self) -> Quaternion<f64> {
        // Note that this is extrinsic rotation.
        Quaternion::from_angle_y(self.yaw) * Quaternion::from_angle_x(self.pitch)
    }

    /// Moves the camera.
    pub fn move_rel(&mut self, vec: Vector3<f64>) {
        self.position += self.camera_direction().rotate_vector(vec);
        trace!("Camera = {:?}", self);
    }

    /// Rotates the camera to up.
    pub fn rotate_up(&mut self, angle: Rad<f64>) {
        self.pitch = (self.pitch + angle).normalize_signed();
        trace!("Camera = {:?}", self);
    }

    /// Rotates the camera to right.
    pub fn rotate_right(&mut self, angle: Rad<f64>) {
        self.yaw = (self.yaw - angle).normalize_signed();
        trace!("Camera = {:?}", self);
    }
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
