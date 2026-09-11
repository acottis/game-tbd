use std::path::Path;

use glam::{Mat4, Quat, Vec3};
use gltf::Node;
use gltf::animation::util::ReadOutputs;
use gltf::{Document, buffer::Data, image::Source, texture::Info};
use image::{DynamicImage, ImageFormat};

use crate::assets::{Material, Mesh, Primitive};
use crate::game::animation::{AnimationId, AnimationSet};
use crate::graphics::Vertex;
use crate::{
    assets::AssetModel,
    game::animation::{AnimationChannel, AnimationClip, AnimationValues},
};

fn load_texture(info: Option<Info>, buffer: &[Data]) -> Option<DynamicImage> {
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

fn load_animations(document: &Document, buffer: &[Data]) -> AnimationSet {
    let mut animation_set = AnimationSet::new();

    for animation in document.animations() {
        let mut channels = Vec::new();
        let mut duration: f32 = 0.0;

        let Some(name) = animation.name() else {
            panic!("Animation with no name!")
        };

        let id = AnimationId::try_from(name).unwrap();

        for channel in animation.channels() {
            let reader = channel.reader(|c| Some(&buffer[c.index()]));

            let times: Vec<f32> = reader.read_inputs().unwrap().collect();

            duration = duration.max(*times.last().unwrap());

            let values = match reader.read_outputs().unwrap() {
                ReadOutputs::Translations(values) => {
                    AnimationValues::Translation(values.map(Vec3::from_array).collect())
                }
                ReadOutputs::Rotations(values) => {
                    AnimationValues::Rotation(values.into_f32().map(Quat::from_array).collect())
                }
                ReadOutputs::Scales(values) => {
                    AnimationValues::Scale(values.map(Vec3::from_array).collect())
                }
                _ => unimplemented!(),
            };

            channels.push(AnimationChannel {
                node: channel.target().node().index(),
                property: channel.target().property(),
                interpolation: channel.sampler().interpolation(),
                times,
                values,
            });
        }
        animation_set.insert(id, AnimationClip { channels, duration });
    }
    animation_set
}

fn load_mesh(meshes: &mut Vec<Mesh>, mesh: gltf::Mesh, transform: Mat4, buffer: &[Data]) {
    let mut primitives = Vec::new();
    for primitive in mesh.primitives() {
        let mut vertex_buffer = Vec::new();
        let mut index_buffer = Vec::new();

        let reader = primitive.reader(|p| Some(&buffer[p.index()]));

        let vertices = reader.read_positions().unwrap();
        let indices = reader.read_indices().unwrap().into_u32();
        let uvs = reader.read_tex_coords(0).unwrap().into_f32();
        if let Some(normals) = reader.read_normals() {
            for ((vertex, uv), normal) in vertices.zip(uvs).zip(normals) {
                vertex_buffer.push(Vertex::new(vertex.into(), normal.into(), uv.into()));
            }
        } else {
            for (vertex, uv) in vertices.zip(uvs) {
                vertex_buffer.push(Vertex::new(vertex.into(), Vec3::Y, uv.into()))
            }
        }

        for index in indices {
            index_buffer.push(index);
        }

        primitives.push(Primitive::new(
            vertex_buffer,
            index_buffer,
            primitive.bounding_box(),
            primitive.material().index(),
        ));
    }
    meshes.push(Mesh {
        primitives,
        transform,
    })
}

fn load_materials(document: &Document, buffer: &[Data]) -> Vec<Material> {
    let mut materials = Vec::new();
    for material in document.materials() {
        let pbr = material.pbr_metallic_roughness();
        let base_colour = pbr.base_color_factor();
        let metallic = pbr.metallic_factor();
        let roughness = pbr.roughness_factor();
        let image = load_texture(pbr.base_color_texture(), &buffer);

        materials.push(Material {
            base_colour,
            metallic,
            roughness,
            image,
        });
    }
    materials
}

fn load_node(meshes: &mut Vec<Mesh>, parent_transform: &Mat4, node: Node, buffer: &[Data]) {
    let transform = parent_transform * Mat4::from_cols_array_2d(&node.transform().matrix());
    if let Some(mesh) = node.mesh() {
        load_mesh(meshes, mesh, transform, buffer);
    }
    for child in node.children() {
        load_node(meshes, &transform, child, buffer);
    }
}

pub fn load(path: impl AsRef<Path>) -> AssetModel {
    let (document, buffer, _) = gltf::import(&path).unwrap();

    // TODO: We only handle one scene?
    let scenes = document.scenes().next().unwrap();

    let mut meshes = Vec::new();
    for node in scenes.nodes() {
        load_node(&mut meshes, &Mat4::IDENTITY, node, &buffer);
    }

    let animations = load_animations(&document, &buffer);
    let materials = load_materials(&document, &buffer);

    AssetModel {
        meshes,
        animations,
        materials,
    }
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
