use std::sync::Arc;

use wgpu::{
    AddressMode, BufferUsages, FilterMode, SamplerDescriptor, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::math::{Mat4, Vec3, Vec4};

pub struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    texture_bind_group: BindGroup,
    camera_bind_group: BindGroup,
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
        let camera = Camera::identity();
        let view = camera.look_at_rh();
        println!("{view:?}");

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&view),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Image stuff
        let image =
            image::load_from_memory(include_bytes!("../assets/ahsoka.jpg"))
                .unwrap();

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
            &image.to_rgba8(),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width() * 4),
                rows_per_image: Some(image.height()),
            },
            size,
        );
        // Sampler
        let texture_view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
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
        let texture_bind_group =
            device.create_bind_group(&BindGroupDescriptor {
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

        let pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&camera_layout, &texture_layout],
                push_constant_ranges: &[],
            });

        // End Texture Stuff

        let shader = device
            .create_shader_module(include_wgsl!("../shaders/shader.wgsl"));

        let render_pipeline =
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: None,
                    compilation_options: Default::default(),
                    buffers: &[VertexBufferLayout {
                        array_stride: size_of::<Vertex2D>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: (size_of::<f32>() * 2) as u64,
                                shader_location: 1,
                            },
                        ],
                    }],
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
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        println!("{:#?}", adapter.get_info());

        Self {
            window,
            surface,
            surface_config,
            device,
            queue,
            render_pipeline,
            texture_bind_group,
            camera_bind_group,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.height = size.height;
        self.surface_config.width = size.width;
        self.surface.configure(&self.device, &self.surface_config);
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
                    load: LoadOp::Clear(Color::WHITE),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        //        let vertices = [
        //            Vertex2D::new(0.0, 0.6, [0., 0.]),
        //            Vertex2D::new(-0.5, 0.1, [0.0, 0.0]),
        //            Vertex2D::new(0.5, 0.1, [0.0, 0.0]),
        //            // Two
        //            Vertex2D::new(0.0, -0.6, [0.0, 0.]),
        //            Vertex2D::new(-0.5, -0.1, [0.0, 0.0]),
        //            Vertex2D::new(0.5, -0.1, [0.0, 0.]),
        //        ];
        let vertices = [
            //            Vertex2D::new(0.0, 0.0, [0.5, 1.0]),
            //            Vertex2D::new(0.0, 1.0, [0.0, 0.0]),
            //            Vertex2D::new(1.0, 0.0, [1.0, 0.0]),
            Vertex2D::new(1.0, 1.0, [0.0, 0.0]),
            Vertex2D::new(-1.0, -1.0, [1.0, 1.0]),
            Vertex2D::new(1.0, -1.0, [0.0, 1.0]),
        ];

        let vertex_buf =
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                usage: BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&vertices),
            });

        // GPU work goes here
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buf.slice(..));
            render_pass.draw(0..3, 0..1);
            //render_pass.draw(3..6, 0..1);
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
        .request_device(&DeviceDescriptor::default())
        .await
        .unwrap();
    (adapter, device, queue)
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
#[repr(C)]
struct Vertex2D {
    x: f32,
    y: f32,
    tex_coords: [f32; 2],
}
impl Vertex2D {
    fn new(x: f32, y: f32, tex_coords: [f32; 2]) -> Self {
        Self { x, y, tex_coords }
    }
}
struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
}

impl Camera {
    fn identity() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            target: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        }
    }
    fn look_at_rh(&self) -> Mat4 {
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
}
