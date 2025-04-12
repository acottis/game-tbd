use std::path::Path;

use gltf::image::Source;
use image::{DynamicImage, ImageFormat};

use super::Vertex3D;

#[derive(Debug)]
pub struct Model3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub images: Vec<DynamicImage>,
}
impl Model3D {
    fn new(
        vertices: Vec<Vertex3D>,
        indices: Vec<u32>,
        images: Vec<DynamicImage>,
    ) -> Self {
        Self {
            vertices,
            indices,
            images,
        }
    }
}

pub fn load_glb(path: impl AsRef<Path>) -> Model3D {
    let (document, buffer, _image) = gltf::import(path).unwrap();

    let mut vertex_buffer = Vec::with_capacity(1000);
    let mut index_buffer = Vec::with_capacity(1000);
    let mut images = Vec::with_capacity(10);

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|p| Some(&buffer[p.index()]));

            let vertices = reader.read_positions().unwrap();
            let uvs = reader.read_tex_coords(0).unwrap().into_f32();
            let indices = reader.read_indices().unwrap().into_u32();

            for (vertex, uv) in vertices.zip(uvs) {
                vertex_buffer.push(Vertex3D::new(vertex.into(), uv))
            }

            for index in indices {
                index_buffer.push(index);
            }
        }
    }

    for texture in document.textures() {
        match texture.source().source() {
            Source::View { view, mime_type } => {
                let parent_buffer_data = &buffer[view.buffer().index()].0;
                let data = &parent_buffer_data
                    [view.offset()..view.offset() + view.length()];
                let mime_type = mime_type.replace('/', ".");
                images.push(
                    image::load_from_memory_with_format(
                        data,
                        ImageFormat::from_path(mime_type).unwrap(),
                    )
                    .unwrap(),
                );
            }
            Source::Uri { .. } => unimplemented!(),
        }
    }

    Model3D::new(vertex_buffer, index_buffer, images)
}
