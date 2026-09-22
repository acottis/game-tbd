use std::path::Path;

use glam::{Mat4, Quat, Vec3};
use gltf::animation::util::ReadOutputs;
use gltf::{Document, buffer::Data, image::Source, texture::Info};
use image::{DynamicImage, ImageFormat};

use crate::assets::{AssetModel, Node, RenderNode};
use crate::assets::{Material, Mesh, Primitive};
use crate::engine::animation::{
    AnimationClip, Joint, NodeAnimation, Rotation, Scale, Skin, Translation,
};
use crate::engine::physics::BoundingBox;
use crate::game::animation::{AnimationId, AnimationSet};
use crate::graphics::Vertex;

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

fn load_skins(document: &Document, buffer: &[Data]) -> Vec<Skin> {
    let mut skins = Vec::with_capacity(document.skins().count());

    for skin in document.skins() {
        // Spec says if inverse bind matrices are missing, use identity.
        let inverse_bind_matrices = match skin
            .reader(|data| Some(&buffer[data.index()]))
            .read_inverse_bind_matrices()
        {
            Some(matrices) => matrices
                .map(|matrix| Mat4::from_cols_array_2d(&matrix))
                .collect(),
            None => vec![Mat4::IDENTITY; skin.joints().len()],
        };

        let joints = skin
            .joints()
            .enumerate()
            .map(|(joint_index, node)| Joint {
                node: node.index() as u32,
                inverse_bind: inverse_bind_matrices[joint_index],
            })
            .collect();
        skins.push(Skin::new(joints));
    }
    skins
}

fn load_animations(document: &Document, buffer: &[Data]) -> AnimationSet {
    let mut animation_set = AnimationSet::new();

    for animation in document.animations() {
        let Some(name) = animation.name() else {
            panic!("Animation with no name!")
        };
        let id = AnimationId::try_from(name).unwrap();

        let mut duration: f32 = 0.0;
        let mut node_animations: Vec<Option<NodeAnimation>> = vec![None; document.nodes().count()];
        for channel in animation.channels() {
            let node = channel.target().node().index();
            let reader = channel.reader(|c| Some(&buffer[c.index()]));

            let times: Vec<f32> = reader.read_inputs().unwrap().collect();
            let interpolation = channel.sampler().interpolation();

            duration = duration.max(*times.last().unwrap());

            let node_animation = node_animations[node].get_or_insert_default();

            match reader.read_outputs().unwrap() {
                ReadOutputs::Translations(values) => {
                    node_animation.translation = Some(Translation::new(
                        interpolation,
                        times,
                        values.map(Vec3::from).collect(),
                    ))
                }
                ReadOutputs::Rotations(values) => {
                    node_animation.rotation = Some(Rotation::new(
                        interpolation,
                        times,
                        values.into_f32().map(Quat::from_array).collect(),
                    ))
                }
                ReadOutputs::Scales(values) => {
                    node_animation.scale = Some(Scale::new(
                        interpolation,
                        times,
                        values.map(Vec3::from).collect(),
                    ))
                }
                _ => unimplemented!(),
            };
        }
        animation_set.insert(id, AnimationClip::new(node_animations, duration));
    }
    animation_set
}

fn load_mesh(mesh: gltf::Mesh, buffer: &[Data]) -> Mesh {
    let mut primitives = Vec::with_capacity(mesh.primitives().count());
    let mut mesh_bbox = BoundingBox::empty();

    for primitive in mesh.primitives() {
        let reader = primitive.reader(|p| Some(&buffer[p.index()]));

        let positions = reader.read_positions().unwrap();
        let uvs = reader.read_tex_coords(0).unwrap().into_f32();
        let mut normals = reader.read_normals();
        let mut joints = reader.read_joints(0).map(|j| j.into_u16());
        let mut weights = reader.read_weights(0).map(|w| w.into_f32());

        let vertices = positions
            .zip(uvs)
            .map(|(position, uv)| {
                let normal = normals
                    .as_mut()
                    .map_or([0.0, 1.0, 0.0], |it| it.next().unwrap());
                let joints = joints.as_mut().map_or([0; 4], |it| it.next().unwrap());
                let weights = weights.as_mut().map_or([0.0; 4], |it| it.next().unwrap());

                Vertex::new(position.into(), normal.into(), uv.into(), joints, weights)
            })
            .collect();

        let indices = reader.read_indices().unwrap().into_u32().collect();

        let gltf_primitive_bbox = primitive.bounding_box();
        let primitive_bbox = BoundingBox {
            min: gltf_primitive_bbox.min.into(),
            max: gltf_primitive_bbox.max.into(),
        };
        mesh_bbox = mesh_bbox.union(primitive_bbox);

        primitives.push(Primitive::new(
            vertices,
            indices,
            primitive.material().index(),
        ));
    }
    Mesh {
        primitives,
        bounding_box: mesh_bbox,
    }
}

fn load_materials(document: &Document, buffer: &[Data]) -> Vec<Material> {
    let mut materials = Vec::with_capacity(document.materials().count());
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

fn load_node(
    nodes: &mut [Node],
    node_order: &mut Vec<u32>,
    render_nodes: &mut Vec<RenderNode>,
    world_transforms: &mut [Mat4],
    bounding_box: &mut BoundingBox,
    node: gltf::Node,
    meshes: &[Mesh],
    parent: Option<u32>,
) {
    let index = node.index();
    node_order.push(index as u32);

    let local_transform = Mat4::from_cols_array_2d(&node.transform().matrix());

    let parent_world_transform = match parent {
        Some(parent) => world_transforms[parent as usize],
        None => Mat4::IDENTITY,
    };
    let world_transform = parent_world_transform * local_transform;

    let mesh_index = if let Some(mesh) = node.mesh() {
        let mesh_index = mesh.index();

        *bounding_box =
            bounding_box.union(meshes[mesh_index].bounding_box.transform(world_transform));

        render_nodes.push(RenderNode {
            node: index as u32,
            mesh: mesh_index as u32,
            skin: node.skin().map(|skin| skin.index() as u32),
        });

        Some(mesh_index as u32)
    } else {
        None
    };

    nodes[index] = Node {
        parent,
        local_transform,
        mesh: mesh_index,
    };
    world_transforms[index] = world_transform;

    for child in node.children() {
        load_node(
            nodes,
            node_order,
            render_nodes,
            world_transforms,
            bounding_box,
            child,
            meshes,
            Some(index as u32),
        );
    }
}

fn load_meshes(document: &Document, buffer: &[Data]) -> Vec<Mesh> {
    document
        .meshes()
        .map(|mesh| load_mesh(mesh, buffer))
        .collect()
}

pub fn load(path: impl AsRef<Path>) -> AssetModel {
    let (document, buffer, _) = gltf::import(&path).unwrap();

    // TODO: We only handle one scene?
    let scenes = document.scenes().next().unwrap();

    let animations = load_animations(&document, &buffer);
    let materials = load_materials(&document, &buffer);
    let skins = load_skins(&document, &buffer);
    let meshes = load_meshes(&document, &buffer);

    let nodes_count = document.nodes().count();

    let mut bounding_box = BoundingBox::empty();
    let mut render_nodes = Vec::with_capacity(nodes_count);
    let mut node_order = Vec::with_capacity(nodes_count);

    let mut nodes = vec![Node::default(); nodes_count];
    let mut rest_world_transforms = vec![Mat4::IDENTITY; nodes_count];

    for node in scenes.nodes() {
        load_node(
            &mut nodes,
            &mut node_order,
            &mut render_nodes,
            &mut rest_world_transforms,
            &mut bounding_box,
            node,
            &meshes,
            None,
        );
    }

    AssetModel {
        nodes,
        meshes,
        animations,
        materials,
        skins,
        bounding_box,
        rest_world_transforms,
        render_nodes,
        node_order,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_assets() {
        load("assets/foo.glb");
        load("assets/ground.glb");
        load("assets/sabine.glb");
    }
}
