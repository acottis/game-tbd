use std::path::Path;

use gltf::{Document, buffer::Data, image::Source, texture::Info};
use image::{DynamicImage, ImageFormat};

use super::{Material, Mesh};
use crate::{
    graphics::{
        Vertex,
        assets::{AnimationChannel, AnimationClip, AnimationValues, AssetModel},
    },
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

fn load_animations(document: &Document, buffer: &Vec<Data>) -> Vec<AnimationClip> {
    let mut animation_clips = Vec::new();
    for animation in document.animations() {
        let mut channels = Vec::new();
        let mut duration: f32 = 0.0;

        for channel in animation.channels() {
            let reader = channel.reader(|c| Some(&buffer[c.index()]));

            let times: Vec<f32> = reader.read_inputs().unwrap().collect();

            duration = duration.max(*times.last().unwrap());

            let values = match reader.read_outputs().unwrap() {
                gltf::animation::util::ReadOutputs::Translations(values) => {
                    AnimationValues::Translation(values.collect())
                }
                gltf::animation::util::ReadOutputs::Rotations(values) => {
                    AnimationValues::Rotation(values.into_f32().collect())
                }
                gltf::animation::util::ReadOutputs::Scales(values) => {
                    AnimationValues::Scale(values.collect())
                }
                _ => unimplemented!(),
            };

            channels.push(AnimationChannel {
                property: channel.target().property(),
                interpolation: channel.sampler().interpolation(),
                times,
                values,
            });
        }
        animation_clips.push(AnimationClip { channels, duration });
    }
    animation_clips
}

fn load_mesh(document: &Document, buffer: &Vec<Data>) -> Vec<Mesh> {
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
    meshes
}

pub fn load(path: impl AsRef<Path>) -> AssetModel {
    let (document, buffer, _) = gltf::import(&path).unwrap();
    let meshes = load_mesh(&document, &buffer);
    let animations = load_animations(&document, &buffer);

    AssetModel { meshes, animations }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_assets() {
        load("assets/foo.glb");
        load("assets/cube.glb");
        load("assets/ground.glb");
    }
}
