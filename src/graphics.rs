use std::{num::NonZeroU64, sync::Arc};

use bytemuck::bytes_of;
use glam::{Mat4, Vec2, Vec3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt as _},
    *,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    assets::{self, AssetModel, AssetModelSet, ModelId},
    engine::Store,
    game::{Entity, Game, light::Light, text::Label},
};

fn animated_transform(entity: &Entity, model: &AssetModel) -> Mat4 {
    let Some(animation) = entity.animation.as_ref() else {
        return entity.transform();
    };

    let Some(clip) = model.animations.get(animation.id) else {
        log::warn!("Clip missing for animation {:?}", animation.id);
        return entity.transform();
    };

    let (translation, rotation, scale) = clip.sample(animation.current_time);

    entity.transform() * Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

fn create_depth_view(device: &Device, size: PhysicalSize<u32>) -> TextureView {
    let depth_texture = device.create_texture(&TextureDescriptor {
        label: Some("Depth Texture"),
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    depth_texture.create_view(&TextureViewDescriptor::default())
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

struct Lighting {
    shadows: Shadows,
    buffer: Buffer,
    layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Lighting {
    fn new(device: &Device, transforms_layout: &BindGroupLayout) -> Self {
        let shadows = Shadows::new(device, transforms_layout);

        let mut bind_group_layout_entires = vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            count: None,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(Light::SIZE as u64),
            },
        }];
        bind_group_layout_entires.extend(shadows.bind_group_layout_entries());
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Lighting"),
            entries: &bind_group_layout_entires,
        });
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Lighting"),
            contents: &[0u8; Light::SIZE],
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let mut bind_group_entries = vec![BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }];
        bind_group_entries.extend(shadows.bind_group_entries());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Lighting"),
            entries: &bind_group_entries,
            layout: &layout,
        });

        Self {
            shadows,
            buffer,
            layout,
            bind_group,
        }
    }
}

struct Shadows {
    view: TextureView,
    camera: GpuTransform,
    pipeline: RenderPipeline,
    sampler: Sampler,
}

impl Shadows {
    const RESOLUTION: u32 = 8192;

    fn new(device: &Device, transforms_layout: &BindGroupLayout) -> Self {
        let camera_layout = GpuTransform::layout(&device, Some("Camera"));
        let camera = GpuTransform::new(&device, &camera_layout, Some("Shadow camera"));

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d {
                width: Self::RESOLUTION,
                height: Self::RESOLUTION,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Sampler"),
            compare: Some(CompareFunction::LessEqual),
            ..Default::default()
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[Some(&camera_layout), Some(&transforms_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(include_wgsl!("../shaders/shadow.wgsl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[Some(Vertex::layout())],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                // Dont render the back of the triangle
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Less),
                stencil: StencilState::default(),
                bias: DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState::default(),
            fragment: None,
            multiview_mask: None,
            cache: None,
        });

        Self {
            view,
            camera,
            pipeline,
            sampler,
        }
    }

    fn bind_group_entries(&self) -> [BindGroupEntry<'_>; 3] {
        [
            BindGroupEntry {
                binding: 1,
                resource: self.camera.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&self.view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::Sampler(&self.sampler),
            },
        ]
    }

    fn bind_group_layout_entries(&self) -> [BindGroupLayoutEntry; 3] {
        [
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(size_of::<Mat4>() as u64),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Depth,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Comparison),
                count: None,
            },
        ]
    }

    fn render_pass_descriptor(&self) -> RenderPassDescriptor<'_> {
        RenderPassDescriptor {
            label: Some("Shadow"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        }
    }
}

struct Text {
    renderer: glyphon::TextRenderer,
    atlas: glyphon::TextAtlas,
    viewport: glyphon::Viewport,
    swash_cache: glyphon::SwashCache,
    font_system: glyphon::FontSystem,
    buffers: Vec<glyphon::Buffer>,
}

impl Text {
    fn new(device: &Device, queue: &Queue, texture_format: TextureFormat) -> Self {
        let font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas = glyphon::TextAtlas::new(&device, &queue, &cache, texture_format);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

        Self {
            renderer,
            atlas,
            viewport,
            swash_cache,
            font_system,
            buffers: Vec::new(),
        }
    }

    fn sync_buffers(&mut self, labels: &Store<Label>) {
        while self.buffers.len() < labels.len() {
            self.buffers.push(glyphon::Buffer::new(
                &mut self.font_system,
                glyphon::Metrics::new(20.0, 32.0),
            ));
        }
    }

    fn update(&mut self, labels: &Store<Label>) {
        for (label, buffer) in labels.iter().zip(self.buffers.iter_mut()) {
            if let Some(label) = label {
                buffer.set_text(
                    &label.text,
                    &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
                    glyphon::Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(&mut self.font_system, false);
            }
        }
    }

    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        labels: &Store<Label>,
    ) {
        let text_areas =
            self.buffers
                .iter_mut()
                .zip(labels.iter())
                .filter_map(move |(buffer, label)| {
                    let label = label.as_ref()?;
                    Some(glyphon::TextArea {
                        buffer,
                        left: label.position.x,
                        top: label.position.y,
                        scale: 1.0,
                        bounds: glyphon::TextBounds {
                            left: 0,
                            top: 0,
                            right: width as i32,
                            bottom: height as i32,
                        },
                        default_color: label.color,
                        custom_glyphs: &[],
                    })
                });

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    fn render_pass_descriptor<'tex>(
        &self,
        color_attachments: &'tex [Option<RenderPassColorAttachment<'tex>>],
    ) -> RenderPassDescriptor<'tex> {
        RenderPassDescriptor {
            label: Some("Glyphon"),
            color_attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        }
    }

    fn resize(&mut self, queue: &Queue, width: u32, height: u32) {
        self.viewport
            .update(&queue, glyphon::Resolution { width, height });
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
#[repr(C)]
struct Transform {
    model: Mat4,
    normal: Mat4,
}

impl Transform {
    fn new(model: Mat4) -> Self {
        Self {
            model,
            normal: model.inverse().transpose(),
        }
    }
}

struct GpuTransform {
    buffer: Buffer,
    bind_group: BindGroup,
}

impl GpuTransform {
    fn new(device: &Device, layout: &BindGroupLayout, label: Option<&str>) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: bytes_of(&Mat4::IDENTITY),
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label,
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    fn write(&mut self, queue: &Queue, transform: &Mat4) {
        queue.write_buffer(&self.buffer, 0, bytes_of(transform));
    }

    fn layout(device: &Device, label: Option<&str>) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(size_of::<Mat4>() as u64),
                },
            }],
        })
    }
}

struct Transforms {
    buffer: Buffer,
    bind_group: BindGroup,
    transforms: Vec<Transform>,
    capacity: usize,
}

impl Transforms {
    fn new(device: &Device, layout: &BindGroupLayout, capacity: usize) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (capacity * size_of::<Transform>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            buffer,
            bind_group,
            capacity,
            transforms: Vec::with_capacity(capacity),
        }
    }

    fn write(
        &mut self,
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        game: &Game,
        asset_models: &AssetModelSet,
    ) {
        // TODO: Think about this
        if self.transforms.len() > self.capacity {
            let transforms = std::mem::take(&mut self.transforms);
            let capacity = transforms.len().next_power_of_two();

            let mut new = Self::new(device, layout, capacity);
            new.transforms = transforms;
            *self = new;
        }

        for entity in &game.entities {
            let model = asset_models.get(entity.model);
            let transform = animated_transform(entity, model);

            for meshes in &model.meshes {
                let model_transform = Transform::new(transform * meshes.transform);
                self.transforms.push(model_transform);
            }
        }
        for object in &game.objects {
            let model = asset_models.get(object.model);
            let transform = object.transform();

            for meshes in &model.meshes {
                let model_transform = Transform::new(transform * meshes.transform);
                self.transforms.push(model_transform);
            }
        }
        let model = asset_models.get(game.terrain.model);
        let transform = game.terrain.transform();
        for meshes in &model.meshes {
            let model_transform = Transform::new(transform * meshes.transform);
            self.transforms.push(model_transform);
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.transforms));
        self.transforms.clear();
    }

    fn layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Model Transforms"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(size_of::<Transform>() as u64),
                },
                count: None,
            }],
        })
    }
}

struct Material {
    bind_group: BindGroup,
}

impl Material {
    fn new(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        material: &assets::Material,
    ) -> Self {
        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let material_uniform = MaterialUniform::from(material);
        let material_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Material Uniform"),
            usage: BufferUsages::UNIFORM,
            contents: bytes_of(&material_uniform),
        });

        let texture_view = if let Some(ref image) = material.image {
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

            texture.create_view(&Default::default())
        } else {
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
            texture.create_view(&Default::default())
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout,
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
        Self { bind_group }
    }

    fn layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(size_of::<MaterialUniform>() as u64),
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
        })
    }
}

struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

impl Model {
    fn load(gpu: &Gpu, model: &AssetModel) -> Self {
        let meshes = model
            .meshes
            .iter()
            .map(|mesh| Mesh::load(&gpu.device, mesh))
            .collect();
        let materials = model
            .materials
            .iter()
            .map(|material| Material::new(&gpu.device, &gpu.queue, &gpu.material_layout, material))
            .collect();
        Self { meshes, materials }
    }

    fn draw(&self, render_pass: &mut RenderPass, transform_index: &mut u32) {
        for mesh in &self.meshes {
            mesh.draw(render_pass, &self.materials, *transform_index);
            *transform_index += 1;
        }
    }

    fn draw_shadow(&self, render_pass: &mut RenderPass, transform_index: &mut u32) {
        for mesh in &self.meshes {
            mesh.draw_shadow(render_pass, *transform_index);
            *transform_index += 1;
        }
    }
}

struct Mesh {
    primitives: Vec<Primitive>,
}

impl Mesh {
    fn load(device: &Device, mesh: &assets::Mesh) -> Self {
        let primitives = mesh
            .primitives
            .iter()
            .map(|primitive| Primitive::load(device, primitive))
            .collect();

        Self { primitives }
    }

    fn draw(&self, render_pass: &mut RenderPass<'_>, materials: &[Material], transform_index: u32) {
        let mut last_material_index = None;

        for primitive in &self.primitives {
            // Don't load material if its already loaded
            if primitive.material_index != last_material_index {
                if let Some(index) = primitive.material_index {
                    render_pass.set_bind_group(2, &materials[index].bind_group, &[]);

                    last_material_index = primitive.material_index;
                }
            }
            render_pass.set_vertex_buffer(0, primitive.vertex.slice(..));
            render_pass.set_index_buffer(primitive.index.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(
                0..primitive.indices_len,
                0,
                transform_index..transform_index + 1,
            );
        }
    }

    fn draw_shadow(&self, render_pass: &mut RenderPass<'_>, transform_index: u32) {
        for primitive in &self.primitives {
            render_pass.set_vertex_buffer(0, primitive.vertex.slice(..));
            render_pass.set_index_buffer(primitive.index.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(
                0..primitive.indices_len,
                0,
                transform_index..transform_index + 1,
            );
        }
    }
}

struct Primitive {
    vertex: Buffer,
    index: Buffer,
    indices_len: u32,
    material_index: Option<usize>,
}

impl Primitive {
    fn load(device: &Device, primitive: &assets::Primitive) -> Self {
        let index = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Model Index Buffer"),
            usage: BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&primitive.indices),
        });
        let vertex = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&primitive.vertices),
        });

        Self {
            vertex,
            index,
            indices_len: primitive.indices.len() as u32,
            material_index: primitive.material,
        }
    }
}

pub struct ModelSet(Vec<Model>);

impl ModelSet {
    pub fn load(gpu: &Gpu, models: &[AssetModel]) -> Self {
        Self(models.iter().map(|model| Model::load(gpu, model)).collect())
    }

    fn get(&self, id: ModelId) -> &Model {
        &self.0[id as usize]
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct MaterialUniform {
    base_colour: [f32; 4],
    metallic: f32,
    roughness: f32,
    has_texture: u32,
    _padding: [u8; 4],
}
impl MaterialUniform {
    fn new(base_colour: [f32; 4], metallic: f32, roughness: f32, has_texture: bool) -> Self {
        Self {
            base_colour,
            metallic,
            roughness,
            has_texture: has_texture as u32,
            _padding: Default::default(),
        }
    }
}

impl From<&assets::Material> for MaterialUniform {
    fn from(m: &assets::Material) -> Self {
        MaterialUniform::new(m.base_colour, m.metallic, m.roughness, m.image.is_some())
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}
impl Vertex {
    const ATTRIBUTES: [VertexAttribute; 3] =
        vertex_attr_array![0 => Float32x3, 1 => Float32x3 ,2 => Float32x2];

    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }

    #[inline(always)]
    pub fn position(&self) -> Vec3 {
        self.position
    }

    const fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Gpu {
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,

    render_pipeline: RenderPipeline,
    depth_view: TextureView,

    camera: GpuTransform,
    transforms: Transforms,
    transforms_layout: BindGroupLayout,
    material_layout: BindGroupLayout,

    lighting: Lighting,
    text: Text,
}

impl Gpu {
    pub fn new(window: Arc<Window>) -> Self {
        let window_size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance.create_surface(window).unwrap();

        let (adapter, device, queue) = pollster::block_on(init_wgpu(&instance, &surface));

        let surface_config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .unwrap();
        surface.configure(&device, &surface_config);

        let mut text = Text::new(&device, &queue, surface_config.format);
        text.resize(&queue, window_size.width, window_size.height);

        let camera_layout = GpuTransform::layout(&device, Some("Camera"));
        let camera = GpuTransform::new(&device, &camera_layout, Some("Camera"));

        let transforms_layout = Transforms::layout(&device);
        let transforms = Transforms::new(&device, &transforms_layout, 64);

        let lighting = Lighting::new(&device, &transforms_layout);
        let material_layout = Material::layout(&device);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&camera_layout),
                Some(&lighting.layout),
                Some(&material_layout),
                Some(&transforms_layout),
            ],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(include_wgsl!("../shaders/main.wgsl"));
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
                // Dont render the back of the triangle
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Less),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let depth_view = create_depth_view(&device, window_size);

        log::info!("{:#?}", adapter.get_info());

        Self {
            surface,
            surface_config,
            device,
            queue,
            render_pipeline,
            material_layout,
            depth_view,
            camera,
            transforms,
            transforms_layout,
            lighting,
            text,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.height = size.height;
        self.surface_config.width = size.width;
        self.surface.configure(&self.device, &self.surface_config);

        self.depth_view = create_depth_view(&self.device, size);

        self.text.resize(&self.queue, size.width, size.height);
    }

    fn render_pass_descriptor<'tex>(
        &'tex self,
        color_attachments: &'tex [Option<RenderPassColorAttachment<'tex>>],
    ) -> RenderPassDescriptor<'tex> {
        RenderPassDescriptor {
            label: Some("Render"),
            color_attachments,
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        }
    }

    pub fn render(
        &mut self,
        window: &Window,
        game: &Game,
        models: &ModelSet,
        asset_models: &AssetModelSet,
    ) {
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            // CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            e => unimplemented!("{e:?}"),
        };

        self.lighting.shadows.camera.write(
            &self.queue,
            &game.light.shadow_transform(game.camera.target()),
        );

        self.camera
            .write(&self.queue, &game.camera.view_projection_matrix());

        self.queue
            .write_buffer(&self.lighting.buffer, 0, bytes_of(&game.light));

        // Transform the models to their wold coordinates
        self.transforms.write(
            &self.device,
            &self.queue,
            &self.transforms_layout,
            game,
            asset_models,
        );

        self.text.sync_buffers(&game.labels);
        self.text.update(&game.labels);
        self.text.prepare(
            &self.device,
            &self.queue,
            self.surface_config.width,
            self.surface_config.height,
            &game.labels,
        );

        let view = &frame.texture.create_view(&Default::default());

        let color_attachments = [Some(RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Default::default()),
                store: StoreOp::Store,
            },
            depth_slice: None,
        })];
        let text_color_attachments = [Some(RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
            depth_slice: None,
        })];

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // Shadow pass
        {
            let mut shadow_pass =
                encoder.begin_render_pass(&self.lighting.shadows.render_pass_descriptor());
            shadow_pass.set_pipeline(&self.lighting.shadows.pipeline);
            shadow_pass.set_bind_group(0, &self.lighting.shadows.camera.bind_group, &[]);
            shadow_pass.set_bind_group(1, &self.transforms.bind_group, &[]);

            let transform_index = &mut 0;
            for entity in &game.entities {
                models
                    .get(entity.model)
                    .draw_shadow(&mut shadow_pass, transform_index);
            }
            for object in &game.objects {
                models
                    .get(object.model)
                    .draw_shadow(&mut shadow_pass, transform_index);
            }
            models
                .get(game.terrain.model)
                .draw(&mut shadow_pass, transform_index);
        }
        // Render the models/light
        {
            let mut render_pass =
                encoder.begin_render_pass(&self.render_pass_descriptor(&color_attachments));
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.lighting.bind_group, &[]);
            render_pass.set_bind_group(3, &self.transforms.bind_group, &[]);

            let transform_index = &mut 0;
            for entity in &game.entities {
                models
                    .get(entity.model)
                    .draw(&mut render_pass, transform_index);
            }
            for object in &game.objects {
                models
                    .get(object.model)
                    .draw(&mut render_pass, transform_index);
            }
            models
                .get(game.terrain.model)
                .draw(&mut render_pass, transform_index);
        }
        // Render 2D text
        {
            let mut render_pass = encoder
                .begin_render_pass(&self.text.render_pass_descriptor(&text_color_attachments));

            self.text
                .renderer
                .render(&self.text.atlas, &self.text.viewport, &mut render_pass)
                .unwrap();
        }
        self.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        self.queue.present(frame);
    }
}
