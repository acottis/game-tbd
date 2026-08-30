use std::path::Path;

use gltf::{buffer::Data, image::Source, texture::Info};
use image::{DynamicImage, ImageFormat};

use super::{Material, Mesh};
use crate::{
    graphics::{Vertex, assets::AssetModel},
    maths::Vec3,
};

fn load_texture(info: Option<Info>, buffer: &Vec<Data>) -> Option<DynamicImage> {
    if let Some(info) = info {
        let image = info.texture().source().source();
        match image {
            Source::View { view, mime_type } => {
                let parent_buffer_data = &buffer[view.buffer().index()].0;
                let data = &parent_buffer_data[view.offset()..view.offset() + view.length()];
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

pub fn inspect_animations(path: impl AsRef<Path>) {
    let (document, buffers, _images) = gltf::import(path).unwrap();

    for animation in document.animations() {
        println!("Animation: {:?}", animation.name());

        for channel in animation.channels() {
            let target = channel.target();

            println!(
                "\nNode {} {:?}",
                target.node().index(),
                target.node().name()
            );

            println!("Path: {:?}", target.property());

            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

            let times = reader.read_inputs().unwrap();
            let outputs = reader.read_outputs().unwrap();

            match outputs {
                gltf::animation::util::ReadOutputs::Translations(values) => {
                    for (time, value) in times.zip(values) {
                        println!("  t={:.3} translation={:?}", time, value);
                    }
                }

                gltf::animation::util::ReadOutputs::Rotations(values) => {
                    for (time, value) in times.zip(values.into_f32()) {
                        println!("  t={:.3} rotation={:?}", time, value);
                    }
                }

                gltf::animation::util::ReadOutputs::Scales(values) => {
                    for (time, value) in times.zip(values) {
                        println!("  t={:.3} scale={:?}", time, value);
                    }
                }

                _ => {}
            }
        }
    }
}
pub fn load_mesh(path: impl AsRef<Path>) -> AssetModel {
    let (document, buffer, _image) = gltf::import(&path).unwrap();

    let mut meshes = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let mut vertex_buffer = Vec::new();
            let mut index_buffer = Vec::new();

            let reader = primitive.reader(|p| Some(&buffer[p.index()]));

            let vertices = reader.read_positions().unwrap();
            let indices = reader.read_indices().unwrap().into_u32();
            let uvs = reader.read_tex_coords(0).unwrap().into_f32();
            if let Some(normals) = reader.read_normals() {
                for ((vertex, uv), normal) in vertices.zip(uvs).zip(normals) {
                    vertex_buffer.push(Vertex::new(vertex.into(), normal.into(), uv));
                }
            } else {
                for (vertex, uv) in vertices.zip(uvs) {
                    vertex_buffer.push(Vertex::new(vertex.into(), Vec3::y(), uv))
                }
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
                    let image = load_texture(pbr.base_color_texture(), &buffer);

                    Material {
                        base_colour,
                        metallic,
                        roughness,
                        image,
                    }
                }
                None => Material::default(),
            };
            meshes.push(Mesh::new(vertex_buffer, index_buffer, material));
        }
    }

    AssetModel(meshes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_animation() {
        inspect_animations("assets/foo.glb");
    }

    #[test]
    fn load_meshes() {
        load_mesh("assets/foo.glb");
        load_mesh("assets/cube.glb");
        load_mesh("assets/ground.glb");
    }
}
