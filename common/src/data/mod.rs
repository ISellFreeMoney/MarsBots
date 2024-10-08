mod vox;

use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use image::{ImageBuffer, Rgba};
use log::info;
use texture_packer::{TexturePacker, TexturePackerConfig};
use crate::{
    block::{Block, BlockMesh, BlockType},
    registry::Registry,
};
use crate::data::vox::{load_voxel_model, VoxelModel};
use crate::item::{Item, ItemMesh, ItemType};

#[derive(Debug, Clone)]
pub struct Data {
    pub blocks: Registry<Block>,
    pub meshes: Vec<BlockMesh>,
    pub texture_atlas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub models: Registry<VoxelModel>,
    pub items: Registry<Item>,
    pub item_meshes: Vec<ItemMesh>,
}

pub fn load_data(data_directory: PathBuf) -> Result<Data> {
    info!("Loading data from {:?}", &data_directory.display());

    let mut textures: Vec<PathBuf> = Vec::new();
    let mut texture_registery: Registry<()> = Default::default();
    let textures_directory = data_directory.join("textures");
    info!("Loading textures from {:?}", &textures_directory.display());
    for dir_entry in std::fs::read_dir(textures_directory).context("couldn't read texture directory")? {
        let dir_entry = dir_entry.context("couldn't read directory entry")?;
        if dir_entry
            .file_type()
            .context("couldn't get file type of directory")?
            .is_file() {
            let file_path = dir_entry.path();

            texture_registery.register(
                file_path
                    .file_stem()
                    .context("couldn't get file stem")?
                    .to_str()
                    .unwrap()
                    .to_owned(),
                (),
            )?;
            textures.push(file_path);
        }
    }
    let (texture_atlas, texture_rects) = load_textures(textures)?;

    let mut models = Registry::default();

    let model_tree = load_voxel_model(
        data_directory.join("model/tree.vox").to_str().unwrap()
    ).unwrap();
    models.register("tree".to_string(), model_tree)?;
    let model_knight = load_voxel_model(
        data_directory.join("model/chr_knight.vox").to_str().unwrap()
    ).unwrap();
    models.register("knight".to_string(), model_knight)?;

    let items_directory = data_directory.join("items");
    let item_datas: Vec<(String, ItemType)> = load_files_from_folder(items_directory);
    let mut items = Registry::default();
    let mut item_meshes = Vec::new();

    for(name, ty) in item_datas.into_iter() {
        match &ty {
            ItemType::NormalItem { texture } => {
                let texture_rect =
                    texture_rects[texture_registery.get_id_by_name(texture).unwrap() as usize];
                let model = self::vox::item::generate_item_model(texture_rect, &texture_atlas);
                let mesh_center = (
                    model.size_x as f32 / 2.0,
                    model.size_y as f32 / 2.0,
                    model.size_z as f32 / 2.0,
                    );
                let scale = 1.0 / usize::max(model.size_x, model.size_y) as f32;
                let mesh_id = models
                    .register(format!("item:{}", name), model)
                    .expect("couldn't register item");
                items
                    .register(name.clone(), Item {name, ty })
                    .expect("couldn't register item");
                item_meshes.push(ItemMesh::SimpleMesh {
                    mesh_id,
                    scale,
                    mesh_center,
                });
            }
        }
    }

    let blocks_directory = data_directory.join("blocks");
    let block_data: Vec<(String, BlockType)> = load_files_from_folder(blocks_directory);

    info!("Processing collected block and texture data");
    let mut blocks = Registry::default();
    let mut meshes = Vec::new();

    blocks
        .register("air".to_owned(),
        Block {
            name: "air".to_owned(),
            block_type: BlockType::Air,
        },
        )
        .expect("couldn't register air block");
    meshes.push(BlockMesh::Empty);

    for(name, block_type) in block_data.into_iter() {
        let block = Block {
            name: name.clone(),
            block_type: block_type.clone(),
        };
        blocks.register(name, block)?;
        let mesh = match block_type {
            BlockType::Air => BlockMesh::Empty,
            BlockType::NormalCube {
                face_texture: names,
            } => BlockMesh::FullCube {
                texture : [
                    texture_rects[texture_registery.get_id_by_name(&names[0]).unwrap() as usize],
                    texture_rects[texture_registery.get_id_by_name(&names[1]).unwrap() as usize],
                    texture_rects[texture_registery.get_id_by_name(&names[2]).unwrap() as usize],
                    texture_rects[texture_registery.get_id_by_name(&names[3]).unwrap() as usize],
                    texture_rects[texture_registery.get_id_by_name(&names[4]).unwrap() as usize],
                    texture_rects[texture_registery.get_id_by_name(&names[5]).unwrap() as usize]
                ],
            },
        };
        meshes.push(mesh);
    }

    info!("Processing block meshes");
    Ok(Data{
        blocks,
        meshes,
        texture_atlas,
        models,
        items,
        item_meshes
    })
}


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextureRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub const MAX_TEXTURE_SIZE: u32 = 2048;

const TEXTURE_PACKER_CONFIG: TexturePackerConfig = TexturePackerConfig {
    max_width: MAX_TEXTURE_SIZE,
    max_height: MAX_TEXTURE_SIZE,
    allow_rotation: false,
    force_max_dimensions: false,
    border_padding: 0,
    texture_padding: 0,
    texture_extrusion: 0,
    trim: false,
    texture_outlines: false,
};

fn load_textures(
    textures: Vec<PathBuf>,
) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, Vec<TextureRect>)> {
    use image::GenericImage;
    use texture_packer::{exporter::ImageExporter, importer::ImageImporter};

    let mut packer = TexturePacker::new_skyline(TEXTURE_PACKER_CONFIG);
    for (i, path) in textures.iter().enumerate() {
        packer.pack_own(
            format!("{}", i),
            ImageImporter::import_from_file(path).expect("Failed to read texture to pack"),
        ).expect("Failed to pack textures");
    }

    let mut texture_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::new(MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE);
    texture_buffer.copy_from(
        &ImageExporter::export(&packer, None).expect("Failed to export texture from packer"),
        0,
        0,
    ).expect("Failed to copy texture atlas to buffer");
    texture_buffer
        .save("atlas.png")
        .expect("Failed to save texture atlas");
    Ok((
        texture_buffer,
        (0..textures.len())
            .map(|i| {
                let frame = packer
                    .get_frame(&format!("{}", i))
                    .expect("Texture packer frame key doesn't exist")
                    .frame;
                TextureRect {
                    x: frame.x as f32 / MAX_TEXTURE_SIZE as f32,
                    y: frame.y as f32 / MAX_TEXTURE_SIZE as f32,
                    width: frame.w as f32 / MAX_TEXTURE_SIZE as f32,
                    height: frame.h as f32 / MAX_TEXTURE_SIZE as f32,
                }
            })
            .collect(),
    ))
}

/// Load all <name>.ron files from a given folder and parse them into type `T`.
fn load_files_from_folder<T: serde::de::DeserializeOwned>(directory: PathBuf) -> Vec<(String, T)> {
    let mut result = Vec::new();
    info!(
        "Loading objects of type {} from directory {}",
        std::any::type_name::<T>(),
        directory.display(),
    );
    for dir_entry in fs::read_dir(directory).expect("Failed to read from directory") {
        let dir_entry = dir_entry.expect("Failed to read directory entry");
        if dir_entry
            .file_type()
            .expect("Failed to get file type")
            .is_file()
        {
            let file_path = dir_entry.path();

            match file_path.extension() {
                None => log::warn!(
                    "No file extension for file {}, skipping...",
                    file_path.display()
                ),
                Some(ext) => {
                    if ext == "ron" {
                        log::info!("Attempting to read file {}", file_path.display());
                        let mut file =
                            fs::File::open(file_path.clone()).expect("Failed to open file");
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer)
                            .expect("Failed to read from file");
                        let file_stem = file_path
                            .file_stem()
                            .expect("Failed to get file stem")
                            .to_str()
                            .unwrap()
                            .to_owned();

                        let parsed_file = {
                            if ext == "ron" {
                                match ron::de::from_str(&buffer) {
                                    Ok(x) => x,
                                    Err(e) => {
                                        log::error!("Failed to parse RON: {}, skipping...", e);
                                        continue;
                                    }
                                }
                            } else {
                                unreachable!("No parser for file format");
                            }
                        };
                        result.push((file_stem, parsed_file));
                    } else {
                        log::warn!("Unsupported file extension {:?}, skipping...", ext);
                        // TODO: display instead of debug
                    }
                }
            }
        }
    }
    result
}
