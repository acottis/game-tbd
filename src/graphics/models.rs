use std::path::Path;

use gltf::{Document, Node, buffer::Data, image::Source, texture::Info};
use image::{DynamicImage, ImageFormat};

use super::Vertex3D;

pub struct Material {
    pub base_colour: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub image: Option<DynamicImage>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_colour: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 1.0,
            image: None,
        }
    }
}

pub struct Model3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub material: Material,
}
impl Model3D {
    fn new(
        vertices: Vec<Vertex3D>,
        indices: Vec<u32>,
        material: Material,
    ) -> Self {
        Self {
            vertices,
            indices,
            material,
        }
    }
}

fn load_texture(
    info: Option<Info>,
    buffer: &Vec<Data>,
) -> Option<DynamicImage> {
    if let Some(info) = info {
        let image = info.texture().source().source();
        match image {
            Source::View { view, mime_type } => {
                let parent_buffer_data = &buffer[view.buffer().index()].0;
                let data = &parent_buffer_data
                    [view.offset()..view.offset() + view.length()];
                let mime_type = mime_type.replace('/', ".");

                image::load_from_memory_with_format(
                    data,
                    ImageFormat::from_path(mime_type).unwrap(),
                )
                .ok()
            }
            Source::Uri { .. } => unimplemented!(),
        }
    } else {
        None
    }
}

fn process_node(
    node: Node,
    document: &Document,
    buffer: &Vec<Data>,
    models: &mut Vec<Model3D>,
) {
    for child in node.children() {
        process_node(child, document, buffer, models);
    }

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let mut vertex_buffer = Vec::with_capacity(1000);
            let mut index_buffer = Vec::with_capacity(1000);

            let reader = primitive.reader(|p| Some(&buffer[p.index()]));

            let vertices = reader.read_positions().unwrap();
            let indices = reader.read_indices().unwrap().into_u32();
            let uvs = reader.read_tex_coords(0).unwrap().into_f32();

            for (vertex, uv) in vertices.zip(uvs) {
                vertex_buffer.push(Vertex3D::new(vertex.into(), uv))
            }

            for index in indices {
                index_buffer.push(index);
            }

            let material = match primitive.material().index() {
                Some(index) => {
                    let material = document.materials().nth(index).unwrap();

                    let pbr = material.pbr_metallic_roughness();
                    let base_colour = pbr.base_color_factor();
                    let metallic = pbr.metallic_factor();
                    let roughness = pbr.roughness_factor();
                    let image = load_texture(pbr.base_color_texture(), buffer);

                    Material {
                        base_colour,
                        metallic,
                        roughness,
                        image,
                    }
                }
                None => Material::default(),
            };
            models.push(Model3D::new(vertex_buffer, index_buffer, material));
        }
    }
}

pub fn load_glb(path: impl AsRef<Path>) -> Vec<Model3D> {
    let (document, buffer, _image) = gltf::import(&path).unwrap();

    let mut models = Vec::new();

    for scene in document.scenes() {
        for node in scene.nodes() {
            process_node(node, &document, &buffer, &mut models);
        }
    }

    models
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        let models = [
            load_glb("assets/BoxTextured.glb"),
            load_glb("assets/cube.glb"),
            load_glb("assets/ground.glb"),
        ];
    }
}
