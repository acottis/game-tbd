use std::{f32::consts::PI, sync::Arc};

use models::Model3D;
use wgpu::{
    BindGroup, Buffer, BufferUsages, IndexFormat, SamplerDescriptor,
    include_wgsl,
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
    models: Vec<GPUModel>,
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
            contents: bytemuck::bytes_of(&camera.view_perspective_rh()),
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
                        min_binding_size: None,
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
                        binding: 1,
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
                    cull_mode: Some(wgpu::Face::Back),
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
        let glb_models = [
            models::load_glb("assets/BoxTextured.glb"),
            models::load_glb("assets/cube.glb"),
        ];
        let mut models = Vec::new();
        for model in glb_models {
            models.push(load_model_into_gpu(
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
        let view =
            &frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder =
            self.device.create_command_encoder(&Default::default());

        let render_pass_desc = RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::GREEN),
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
            bytemuck::bytes_of(&self.camera.view_perspective_rh()),
        );

        // GPU work goes here
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for model in &self.models {
                render_pass.set_bind_group(1, &model.texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, model.vertex.slice(..));
                render_pass.set_index_buffer(
                    model.index.slice(..),
                    IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..model.indices_size, 0, 0..1);
            }
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        frame.present();
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

pub struct GPUModel {
    pub vertex: Buffer,
    pub index: Buffer,
    pub indices_size: u32,
    pub texture_bind_group: BindGroup,
}

fn load_model_into_gpu(
    device: &Device,
    queue: &Queue,
    texture_layout: &BindGroupLayout,
    model: &Model3D,
) -> GPUModel {
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

    let image = model.images.first().unwrap().to_rgba8();

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

    // Sampler
    let texture_view = texture.create_view(&Default::default());
    let sampler = device.create_sampler(&SamplerDescriptor::default());
    let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Texture Bind Group"),
        layout: &texture_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
        ],
    });

    GPUModel {
        vertex,
        index,
        indices_size: model.indices.len() as u32,
        texture_bind_group,
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct Vertex3D {
    vec3: Vec3,
    uv: [f32; 2],
}
impl Vertex3D {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

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
            position: Vec3::new(0.0, 1.0, 2.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fovy: PI / 4.0,
            aspect: window_size.width as f32 / window_size.height as f32,
            near: 0.1,
            far: 100.0,
        }
    }
    pub fn move_xy(&mut self, dx: f32, dy: f32) {
        self.forward(dy);
        self.strafe(dx);
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
    #[inline(always)]
    pub fn strafe_right(&mut self, dx: f32) {
        self.strafe(dx);
    }
    #[inline(always)]
    pub fn strafe_left(&mut self, dx: f32) {
        self.strafe(-dx);
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
    pub fn view_perspective_rh(&self) -> [Mat4; 2] {
        [self.view_rh(), self.perspective_rh()]
    }

    fn set_aspect_ratio(&mut self, size: &PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32
    }
}
