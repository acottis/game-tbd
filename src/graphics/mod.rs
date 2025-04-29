use std::{num::NonZeroU64, rc::Rc, sync::Arc};

use assets::{MaterialUniform, Mesh, Vertex, load_glb};
use bytemuck::{Pod, Zeroable, bytes_of};
use camera::Camera;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    game::Entity,
    math::{Mat4, Vec3},
};

mod assets;
mod camera;

pub struct State {
    pub window: Arc<Window>,
    pub camera: Camera,
    pub meshes: Vec<Rc<MeshInstance>>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    texture_layout: BindGroupLayout,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    light_bind_group: BindGroup,
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
                    visibility: ShaderStages::VERTEX,
                    count: None,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            size_of::<Mat4>() as _
                        ),
                    },
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

        let light =
            Light::new(Vec3::new(0.0, 0.5, 0.5), Vec3::new(1.0, 1.0, 0.0), 0.5);
        let light_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Light"),
            contents: bytes_of(&light),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let light_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Light"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            size_of::<Light>() as _
                        ),
                    },
                }],
            });
        let light_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light"),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            layout: &light_layout,
        });

        let shader = device
            .create_shader_module(include_wgsl!("../../shaders/shader.wgsl"));

        let texture_layout_descriptor = BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            size_of::<Mat4>() as u64
                        ),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
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
                    binding: 2,
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
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };
        let texture_layout =
            device.create_bind_group_layout(&texture_layout_descriptor);

        let pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &camera_layout,
                    &light_layout,
                    &texture_layout,
                ],
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
                    buffers: &[Vertex::layout()],
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

        println!("{:#?}", adapter.get_info());

        Self {
            window,
            surface,
            surface_config,
            device,
            queue,
            texture_layout,
            render_pipeline,
            camera,
            camera_bind_group,
            camera_buffer,
            light_bind_group,
            meshes: Vec::new(),
        }
    }

    pub fn load_models(&mut self) {
        [
            load_glb("assets/BoxTextured.glb"),
            load_glb("assets/cube.glb"),
            load_glb("assets/ground.glb"),
        ]
        .into_iter()
        .flatten()
        .for_each(|model| self.meshes.push(Rc::new(self.load_model(&model))));
    }
    fn load_model(&self, model: &Mesh) -> MeshInstance {
        let index = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&model.indices),
        });
        let vertex = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&model.vertices),
        });

        let sampler = self.device.create_sampler(&SamplerDescriptor::default());

        let material_uniform = MaterialUniform::from(&model.material);
        let material_uniform_buffer =
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                usage: BufferUsages::UNIFORM,
                contents: bytes_of(&material_uniform),
            });

        let vertex_uniform =
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Uniform"),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                contents: bytes_of(&Mat4::identity()),
            });

        let texture_view = if let Some(ref image) = model.material.image {
            let image = image.to_rgba8();
            let size = Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            };
            let texture = self.device.create_texture(&TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.queue.write_texture(
                texture.as_image_copy(),
                &image,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image.width() * 4),
                    rows_per_image: Some(image.height()),
                },
                size,
            );

            texture.create_view(&Default::default())
        } else {
            let texture = self.device.create_texture(&TextureDescriptor {
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
            texture.create_view(&Default::default())
        };

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &self.texture_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: vertex_uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: material_uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        MeshInstance {
            vertex,
            index,
            indices_len: model.indices.len() as u32,
            bind_group,
            vertex_uniform,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.height = size.height;
        self.surface_config.width = size.width;
        self.surface.configure(&self.device, &self.surface_config);
        self.camera.set_aspect_ratio(&size);
    }

    pub fn render(&mut self, entities: &Vec<Entity>) {
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
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);

            for entity in entities {
                entity.mesh.transform(&self.queue, entity.transform());
                render_pass.set_bind_group(2, &entity.mesh.bind_group, &[]);
                render_pass.set_vertex_buffer(0, entity.mesh.vertex.slice(..));
                render_pass.set_index_buffer(
                    entity.mesh.index.slice(..),
                    IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..entity.mesh.indices_len, 0, 0..1);
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

pub struct MeshInstance {
    vertex: Buffer,
    index: Buffer,
    indices_len: u32,
    bind_group: BindGroup,
    vertex_uniform: Buffer,
}

impl MeshInstance {
    fn transform(&self, queue: &Queue, matrix: Mat4) {
        queue.write_buffer(&self.vertex_uniform, 0, bytes_of(&matrix));
    }
}

#[derive(Zeroable, Pod, Copy, Clone)]
#[repr(C)]
pub struct Light {
    position: Vec3,
    _padding: [u8; 4],
    color: Vec3,
    intensity: f32,
}

impl Light {
    fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            _padding: Default::default(),
        }
    }
}
