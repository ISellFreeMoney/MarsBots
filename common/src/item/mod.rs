use serde::Deserialize;

pub type ItemId = u32;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "Item")]
pub enum ItemType {
    NormalItem { texture: String },
}

#[derive(Debug, Clone)]
pub enum ItemMesh {
    SimpleMesh {
        mesh_id: u32,
        scale: f32,
        mesh_center: (f32, f32, f32),
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub ty: ItemType,
}