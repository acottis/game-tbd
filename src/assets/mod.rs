mod gltf;

use std::path::Path;

use ::gltf::mesh::BoundingBox;
use glam::Mat4;
pub use gltf::load;
use image::DynamicImage;

use crate::{game::animation::AnimationSet, graphics::Vertex};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ModelId {
    Foo = 0,
    BoxTextured,
    Cube,
    Platform,
    Ground,
}

impl ModelId {
    pub const COUNT: usize = 5;
}

impl TryFrom<&Path> for ModelId {
    type Error = String;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let Some(stem) = path.file_stem() else {
            return Err(format!("invalid model filename: {}", path.display()));
        };

        let Some(name) = stem.to_str() else {
            return Err(format!(
                "model filename is not valid UTF-8: {}",
                path.display()
            ));
        };

        match name {
            "foo" => Ok(Self::Foo),
            "BoxTextured" => Ok(Self::BoxTextured),
            "cube" => Ok(Self::Cube),
            "platform" => Ok(Self::Platform),
            "ground" => Ok(Self::Ground),
            _ => Err(format!("unknown model '{name}': {}", path.display())),
        }
    }
}

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

pub struct AssetModel {
    pub meshes: Vec<Mesh>,
    pub animations: AnimationSet,
    pub materials: Vec<Material>,
}
pub struct AssetModelSet(pub Vec<AssetModel>);

impl AssetModelSet {
    pub fn load() -> Self {
        let dir = std::fs::read_dir("assets").unwrap();

        let mut assets: Vec<Option<AssetModel>> = (0..ModelId::COUNT).map(|_| None).collect();
        for entry in dir {
            let path = entry.unwrap().path();

            if path.extension().and_then(|ext| ext.to_str()) != Some("glb") {
                continue;
            }

            let model_id = ModelId::try_from(path.as_path()).unwrap();

            assets[model_id as usize] = Some(load(&path));
        }

        let assets = assets.into_iter().map(Option::unwrap).collect();
        Self(assets)
    }

    pub fn get(&self, id: ModelId) -> &AssetModel {
        &self.0[id as usize]
    }
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub transform: Mat4,
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounding_box: BoundingBox,
    pub material: Option<usize>,
}

impl Primitive {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        bounding_box: BoundingBox,
        material: Option<usize>,
    ) -> Self {
        Self {
            vertices,
            indices,
            bounding_box,
            material,
        }
    }
}
