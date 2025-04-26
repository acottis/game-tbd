use std::{
    default, f32::consts::PI, num::NonZeroU64, sync::Arc, time::Instant,
};

use bytemuck::bytes_of;
use models::Model3D;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::math::{Mat3, Mat4, Vec3, Vec4};

mod models;

pub struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    pub camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    models: Vec<GpuModel>,
    last_frame_time: Instant,
}

impl State {
    pub fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let window_size = window.inner_size();
        let instance =
            Instance::new(&InstanceDescriptor::from_env_or_default());
        let surface = instance.create_surface(window.clone()).unwrap();

        let (adapter, device, queue) =
            pollster::block_on(init_wgpu(&instance, &surface));

        let surface_config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .unwrap();
        surface.configure(&device, &surface_config);

        // Camera stuff
        let camera = Camera::new(&window_size);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(&camera.view_perspective_rh()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let camera_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            size_of::<CameraUniform>() as u64,
                        ),
                    },
                    visibility: ShaderStages::VERTEX,
                    count: None,
                }],
            });
        let camera_bind_group =
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        let shader = device
            .create_shader_module(include_wgsl!("../../shaders/shader.wgsl"));

        let texture_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(size_of::<
                                MaterialUniform,
                            >(
                            )
                                as u64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&camera_layout, &texture_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: None,
                    compilation_options: Default::default(),
                    buffers: &[Vertex3D::layout()],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: None,
                    compilation_options: Default::default(),
                    targets: &[Some(surface_config.format.into())],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        // Load models
        let glb_models: Vec<Model3D> = [
            models::load_glb("assets/BoxTextured.glb"),
            models::load_glb("assets/cube.glb"),
            models::load_glb("assets/ground.glb"),
        ]
        .into_iter()
        .flatten()
        .collect();
        let mut models = Vec::new();
        for model in glb_models {
            models.push(GpuModel::new(
                &device,
                &queue,
                &texture_layout,
                &model,
            ));
        }

        println!("{:#?}", adapter.get_info());

        Self {
            window,
            surface,
            surface_config,
            device,
            queue,
            render_pipeline,
            camera,
            camera_bind_group,
            camera_buffer,
            models,
            last_frame_time: Instant::now(),
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.height = size.height;
        self.surface_config.width = size.width;
        self.surface.configure(&self.device, &self.surface_config);
        self.camera.set_aspect_ratio(&size);
    }

    pub fn render(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = &frame.texture.create_view(&Default::default());

        let mut encoder =
            self.device.create_command_encoder(&Default::default());

        let render_pass_desc = RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Default::default()),
                    // WARNING: This is important to vulkan but not dx12
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytes_of(&self.camera.view_perspective_rh()),
        );

        // GPU work goes here
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for model in &self.models {
                render_pass.set_bind_group(1, &model.bind_group, &[]);
                render_pass.set_vertex_buffer(0, model.vertex.slice(..));
                render_pass.set_index_buffer(
                    model.index.slice(..),
                    IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..model.indices_len, 0, 0..1);
            }
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        frame.present();
        let now = Instant::now();
        let dt = Instant::now()
            .duration_since(self.last_frame_time)
            .as_secs_f64();
        println!("FPS: {}", 1.0 / dt);
        self.last_frame_time = now;
    }
}

async fn init_wgpu(
    instance: &Instance,
    surface: &Surface<'static>,
) -> (Adapter, Device, Queue) {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: Default::default(),
            compatible_surface: Some(surface),
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::default(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        })
        .await
        .unwrap();
    (adapter, device, queue)
}

struct GpuModel {
    vertex: Buffer,
    index: Buffer,
    indices_len: u32,
    bind_group: BindGroup,
}

impl GpuModel {
    fn new(
        device: &Device,
        queue: &Queue,
        texture_layout: &BindGroupLayout,
        model: &Model3D,
    ) -> GpuModel {
        let index = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&model.indices),
        });
        let vertex = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&model.vertices),
        });

        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let bind_group = if let Some(ref image) = model.material.image {
            let material_uniform =
                MaterialUniform::new(model.material.base_colour, true);
            let material_uniform_buffer =
                device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    usage: BufferUsages::UNIFORM,
                    contents: bytes_of(&material_uniform),
                });

            let image = image.to_rgba8();
            let size = Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            queue.write_texture(
                texture.as_image_copy(),
                &image,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image.width() * 4),
                    rows_per_image: Some(image.height()),
                },
                size,
            );

            let texture_view = texture.create_view(&Default::default());
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Texture Bind Group"),
                layout: &texture_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: material_uniform_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            })
        } else {
            let material_uniform =
                MaterialUniform::new(model.material.base_colour, false);

            let material_uniform_buffer =
                device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    usage: BufferUsages::UNIFORM,
                    contents: bytes_of(&material_uniform),
                });

            let texture = device.create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let texture_view = texture.create_view(&Default::default());

            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Texture Bind Group"),
                layout: &texture_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: material_uniform_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            })
        };
        GpuModel {
            vertex,
            index,
            indices_len: model.indices.len() as u32,
            bind_group,
        }
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct Vertex3D {
    vec3: Vec3,
    uv: [f32; 2],
}
impl Vertex3D {
    const ATTRIBUTES: [VertexAttribute; 2] =
        vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn new(vec3: Vec3, uv: [f32; 2]) -> Self {
        Self { vec3, uv }
    }

    const fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct MaterialUniform {
    base_colour: [f32; 4],
    has_texture: u32,
    _padding: [u8; 12],
}

impl MaterialUniform {
    fn new(base_colour: [f32; 4], has_texture: bool) -> Self {
        Self {
            has_texture: has_texture as _,
            base_colour,
            _padding: Default::default(),
        }
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct CameraUniform {
    view: Mat4,
    projection: Mat4,
}

#[derive(Debug)]
pub struct Camera {
    /// Our position (eye)
    position: Vec3,
    /// The center of what we are looking at, rotations are relative to target
    target: Vec3,
    up: Vec3,
    /// Field of view
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl Camera {
    const fn new(window_size: &PhysicalSize<u32>) -> Self {
        Self {
            position: Vec3::new(0.5, 1.0, 2.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fovy: PI / 4.0,
            aspect: window_size.width as f32 / window_size.height as f32,
            near: 0.1,
            far: 1000.0,
        }
    }
    pub fn rotate_x(&mut self, theta: f32) {
        self.position = Mat3::rotation_x(theta) * self.position;
    }
    pub fn rotate_y(&mut self, theta: f32) {
        self.position = Mat3::rotation_y(theta) * self.position;
    }
    pub fn forward(&mut self, speed: f32) {
        let forward = (self.target - self.position).normalise();

        self.position += forward * speed;
    }
    /// + is right
    /// - is left
    pub fn strafe(&mut self, speed: f32) {
        let forward = (self.target - self.position).normalise();
        let right = forward.cross(&self.up).normalise();
        let delta = right * speed;

        self.position += delta;
        self.target += delta;
    }
    fn view_rh(&self) -> Mat4 {
        let forward = (self.target - self.position).normalise();
        let right = forward.cross(&self.up).normalise();
        let up = right.cross(&forward).normalise();

        let projection_x = -right.dot(&self.position);
        let projection_y = -up.dot(&self.position);
        let projection_z = forward.dot(&self.position);

        Mat4 {
            x: Vec4::new(right.x, up.x, -forward.x, 0.0),
            y: Vec4::new(right.y, up.y, -forward.y, 0.0),
            z: Vec4::new(right.z, up.z, -forward.z, 0.0),
            w: Vec4::new(projection_x, projection_y, projection_z, 1.0),
        }
    }
    fn perspective_rh(&self) -> Mat4 {
        let tan_half_fov = 1.0 / (self.fovy / 2.0).tan();
        let range = self.far - self.near;
        let depth = -(self.far + self.near) / range;
        let project = -(2.0 * self.far * self.near) / range;
        Mat4 {
            x: Vec4::new(tan_half_fov / self.aspect, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, tan_half_fov, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, depth, -1.0),
            w: Vec4::new(0.0, 0.0, project, 0.0),
        }
    }
    fn view_perspective_rh(&self) -> CameraUniform {
        CameraUniform {
            view: self.view_rh(),
            projection: self.perspective_rh(),
        }
    }

    fn set_aspect_ratio(&mut self, size: &PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32
    }
}
