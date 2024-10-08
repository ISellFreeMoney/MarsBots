use serde::Deserialize;

pub type BlockId = u16;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "Block")]
pub enum BlockType {
    Air,
    NormalCube { face_texture: Vec<String>},
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: String,
    pub block_type: BlockType,
}

#[derive(Debug, Clone)]
pub enum BlockMesh {
    Empty,
    FullCube { texture: [TextureRect; 6] },
}

impl BlockMesh  {
    pub fn is_opaque(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::FullCube { .. } => true,
        }
    }
}