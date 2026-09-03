use std::num::NonZeroU64;

use bytemuck::bytes_of;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt as _},
    *,
};
use winit::window::Window;

use crate::{
    game::{Entity, ModelId},
    graphics::assets::AssetModel,
    maths::{Mat4, Vec3},
};

use super::{Camera, Light, assets};

pub struct Gpu {
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    texture_layout: BindGroupLayout,
    transform_layout: BindGroupLayout,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    light_bind_group: BindGroup,
    _light_buffer: Buffer,
}

impl Gpu {
    pub fn new(
        window: impl Into<SurfaceTarget<'static>>,
        window_width: u32,
        window_height: u32,
        camera: &Camera,
        light: &Light,
    ) -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance.create_surface(window).unwrap();

        let (adapter, device, queue) = pollster::block_on(init_wgpu(&instance, &surface));

        let surface_config = surface
            .get_default_config(&adapter, window_width, window_height)
            .unwrap();
        surface.configure(&device, &surface_config);

        let (camera_bind_group, camera_buffer, camera_layout) = load_camera(&device, camera);

        let (light_bind_group, _light_buffer, light_layout) = load_light(&device, light);

        let texture_layout = texture_layout(&device);
        let transform_layout = transform_layout(&device);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&camera_layout),
                Some(&light_layout),
                Some(&texture_layout),
                Some(&transform_layout),
            ],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(include_wgsl!("../../shaders/shader.wgsl"));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[Some(Vertex::layout())],
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
            cache: None,
            multiview_mask: None,
        });

        log::info!("{:#?}", adapter.get_info());

        Self {
            surface,
            surface_config,
            device,
            queue,
            texture_layout,
            transform_layout,
            render_pipeline,
            camera_bind_group,
            camera_buffer,
            light_bind_group,
            _light_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.height = height;
        self.surface_config.width = width;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn render(
        &mut self,
        window: &Window,
        entities: &[Entity],
        camera: &Camera,
        models: &GpuModels,
    ) {
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            e => unimplemented!("{e:?}"),
        };
        let view = &frame.texture.create_view(&Default::default());

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
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // GPU work goes here
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&self.render_pipeline);

            // Update camera buffer
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytes_of(&camera.view_perspective_rh()),
            );
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);

            for entity in entities {
                let transform =
                    Mat4::from_translation(entity.position()) * Mat4::from_scaling(entity.scale());
                // Update entity buffer
                self.queue
                    .write_buffer(&entity.transform.buffer, 0, bytes_of(&transform));

                let model = models.get(entity.model);
                render_pass.set_bind_group(2, &model.bind_group, &[]);
                render_pass.set_bind_group(3, &entity.transform.bind_group, &[]);

                render_pass.set_vertex_buffer(0, model.vertex.slice(..));

                render_pass.set_index_buffer(model.index.slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..model.indices_len, 0, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        self.queue.present(frame);
    }
}

fn load_camera(device: &Device, camera: &Camera) -> (BindGroup, Buffer, BindGroupLayout) {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytes_of(&camera.view_perspective_rh()),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let min_binding_size = NonZeroU64::new(size_of::<Mat4>() as u64);
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Camera"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            count: None,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size,
            },
        }],
    });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Camera"),
        layout: &layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (bind_group, buffer, layout)
}

fn load_light(device: &Device, light: &Light) -> (BindGroup, Buffer, BindGroupLayout) {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Light"),
        contents: bytes_of(light),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let min_binding_size = NonZeroU64::new(size_of::<Light>() as u64);
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Light"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            count: None,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size,
            },
        }],
    });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Light"),
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        layout: &layout,
    });

    (bind_group, buffer, layout)
}

fn texture_layout(device: &Device) -> BindGroupLayout {
    let min_binding_size = NonZeroU64::new(size_of::<MaterialUniform>() as u64);
    let layout_descriptor = BindGroupLayoutDescriptor {
        label: Some("Texture"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
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
    };
    device.create_bind_group_layout(&layout_descriptor)
}

fn transform_layout(device: &Device) -> BindGroupLayout {
    let min_binding_size = NonZeroU64::new(size_of::<Mat4>() as u64);
    let transform_layout_descriptor = BindGroupLayoutDescriptor {
        label: Some("Transform"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size,
            },
            count: None,
        }],
    };
    device.create_bind_group_layout(&transform_layout_descriptor)
}

async fn init_wgpu(instance: &Instance, surface: &Surface<'static>) -> (Adapter, Device, Queue) {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: Default::default(),
            compatible_surface: Some(surface),
            apply_limit_buckets: false,
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
            experimental_features: ExperimentalFeatures::disabled(),
        })
        .await
        .unwrap();
    (adapter, device, queue)
}

pub struct ModelTransform {
    buffer: Buffer,
    bind_group: BindGroup,
}

impl ModelTransform {
    pub fn new(gpu: &Gpu) -> Self {
        let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Transform"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: bytes_of(&Mat4::identity()),
        });
        let bind_group = gpu.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Transform"),
            layout: &gpu.transform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }
}

pub struct GpuModel {
    vertex: Buffer,
    index: Buffer,
    indices_len: u32,
    bind_group: BindGroup,
}

impl GpuModel {
    pub fn load(gpu: &Gpu, model: &assets::AssetModel) -> Self {
        // TODO: Handle more than one mesh
        let model = model.meshes.first().unwrap();

        let index = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&model.indices),
        });
        let vertex = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&model.vertices),
        });

        let sampler = gpu.device.create_sampler(&SamplerDescriptor::default());

        let material_uniform = MaterialUniform::from(&model.material);
        let material_uniform_buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM,
            contents: bytes_of(&material_uniform),
        });

        let texture_view = if let Some(ref image) = model.material.image {
            let image = image.to_rgba8();
            let size = Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            };
            let texture = gpu.device.create_texture(&TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            gpu.queue.write_texture(
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
            let texture = gpu.device.create_texture(&TextureDescriptor {
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

        let bind_group = gpu.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &gpu.texture_layout,
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
        });
        GpuModel {
            vertex,
            index,
            indices_len: model.indices.len() as u32,
            bind_group,
        }
    }
}

pub struct GpuModels([GpuModel; 3]);

impl GpuModels {
    pub fn load(gpu: &Gpu, asset_models: [AssetModel; 3]) -> Self {
        Self(asset_models.map(|model| GpuModel::load(gpu, &model)))
    }

    pub fn get(&self, id: ModelId) -> &GpuModel {
        &self.0[id as usize]
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
pub struct MaterialUniform {
    base_colour: [f32; 4],
    metallic: f32,
    roughness: f32,
    has_texture: u32,
    _padding: [u8; 4],
}
impl MaterialUniform {
    pub fn new(base_colour: [f32; 4], metallic: f32, roughness: f32, has_texture: bool) -> Self {
        Self {
            base_colour,
            metallic,
            roughness,
            has_texture: has_texture as u32,
            _padding: Default::default(),
        }
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    vec3: Vec3,
    normal: Vec3,
    uv: [f32; 2],
}
impl Vertex {
    const ATTRIBUTES: [VertexAttribute; 3] =
        vertex_attr_array![0 => Float32x3, 1 => Float32x3 ,2 => Float32x2];

    pub fn new(vec3: Vec3, normal: Vec3, uv: [f32; 2]) -> Self {
        Self { vec3, normal, uv }
    }

    const fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
