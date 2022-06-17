use std::ops::{Index, IndexMut};

use bevy::math::Vec3A;
use bevy::pbr::wireframe::WireframeConfig;
use bevy::render::mesh::{PrimitiveTopology, VertexAttributeValues, Indices};
use bevy::{prelude::*, utils::HashMap};
use enum_map::{EnumMap, Enum};
use ndarray::{Array3, Shape, Dim, Array};
use block_mesh::ndshape::{ConstShape, ConstShape3u32, ConstShape3usize};
use block_mesh::{greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};


// Consts
const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 16;
const CHUNK_LENGTH: usize = 16;


// Plugin
#[derive(Default)]
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
         .add_event::<SetBlockEvent>()
         .init_resource::<LoadedChunks>();

    }
}


// Events

pub struct SetBlockEvent {
    shape: SetBlockShape,
    block: Block,
}

// Systems
pub fn map_setup (
    mut wireframe_config: ResMut<WireframeConfig>,
    mut ev_set_block: EventWriter<SetBlockEvent>,
) {
    wireframe_config.global = true;

    //create a 64x16x64 grid of blocks
    //save this to a chunks resource thingy.
    let dirt = Block::new(BlockType::Dirt);

    for x in -2..2 {
        for z in -2..2 {
            ev_set_block.send(SetBlockEvent { shape: SetBlockShape::Chunk(IVec3::new(x, 0, z)), block: dirt })
        }
    }
    

}

pub fn set_block_chunk (
    mut chunks: ResMut<LoadedChunks>,

    mut ev_set_block_chunk: EventReader<SetBlockEvent>,

    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in ev_set_block_chunk.iter() {
        match ev.shape {
            SetBlockShape::Chunk(location) => {
                chunks.set_block_chunk(location, ev.block, &mut commands, &mut meshes, &mut materials);
            }

            _ => {
                // TODO: add the other match arms lol
                println!("haha wait shit i wasn't ready");
            }

        }


        
    }
}

// Helper functions
// Yoinked from block-mesh example cause I can't be assed.
fn generate_greedy_mesh(
    meshes: &mut Assets<Mesh>,
    chunk: &Array3<Block>,
) -> Handle<Mesh> {
    type SampleShape = ConstShape3u32<{ CHUNK_WIDTH as u32 + 2 }, { CHUNK_HEIGHT as u32 + 2 }, { CHUNK_LENGTH as u32 + 2 }>;

    
    //let mut samples = [Block::new(BlockType::Air); SampleShape::SIZE as usize];
    
    let mut samples = Array::from_elem((CHUNK_WIDTH + 2, CHUNK_HEIGHT + 2, CHUNK_LENGTH + 2), Block::new(BlockType::Air));

    for ((x, y, z), block) in chunk.indexed_iter() {
        samples[(x + 1, y + 1, z + 1)] = *block;
    }
    
    //println!("{:?}", samples.as_slice().unwrap());

    //for i in 0u32..(SampleShape::SIZE) {
    //    let index = SampleShape::delinearize(i);
        
        //let p = into_domain(16, SampleShape::delinearize(i));
        //samples[i as usize] = chunk[bevy::math::Vec3A::from()];
    //}
    

    //let mut samples = chunk.as_slice().unwrap();
    //samples = [[], samples, []]

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new((CHUNK_WIDTH + 2) * (CHUNK_HEIGHT + 2) * (CHUNK_LENGTH + 2));
    greedy_quads(
        samples.as_slice().unwrap(),
        &SampleShape {},
        [0; 3],
        [CHUNK_WIDTH as u32 + 2 - 1, CHUNK_HEIGHT as u32 + 2 - 1, CHUNK_LENGTH as u32 + 2 - 1],
        &faces,
        &mut buffer,
    );

    println!("{:?}", buffer.quads.num_quads());

    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]));
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    let mesh = meshes.add(render_mesh);
    

    mesh
}

// Yoinked as above too.
//fn into_domain(array_dim: u32, [x, y, z]: [u32; 3]) -> Vec3A {
//    (2.0 / array_dim as f32) * Vec3A::new(x as f32, y as f32, z as f32) - 1.0
//}

// Data
pub enum SetBlockShape {
    Block(IVec3),
    Chunk(IVec3),
    Range(IVec3),
}

#[derive(PartialEq, Eq, Clone)]
pub enum BlockMaterial {
    None,
    Single{texture: Handle<StandardMaterial>},
    Three{top: Handle<StandardMaterial>, sides: Handle<StandardMaterial>, bottom: Handle<StandardMaterial>},

}
impl Default for BlockMaterial {
    fn default() -> Self {
        BlockMaterial::None
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Block {
    block_type: BlockType,
    damage: f32,
    // This will have the type of as well as the amount of material dropped per hit
    // It should be in the enum, not here.
    //material_drop:
}
impl Block {
    pub fn new (block_type: BlockType) -> Self {
        Self {block_type, damage: 0.0}
    }
    pub fn new_with_damage (block_type: BlockType, damage: f32) -> Self {
        Self {block_type, damage}
    }
}
impl Voxel for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        self.block_type.visibility()
    }
}
impl MergeVoxel for Block {
    type MergeValue = BlockType;

    fn merge_value(&self) -> Self::MergeValue {
        self.block_type
    }
}

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum BlockType {
    Air,
    Dirt,
}
impl Default for BlockType {
    fn default() -> Self {
        BlockType::Air
    }
}
impl BlockType {
    pub fn visibility(&self) -> VoxelVisibility {
        match self {
            Air => {VoxelVisibility::Empty}
            Dirt => {VoxelVisibility::Opaque}

            _ => {VoxelVisibility::Opaque}
        }
    }
}

pub struct Chunk {
    blocks: Array3<Block>,
    entity: Entity,
}
impl Chunk {
    pub fn new(blocks: Array3<Block>, entity: Entity) -> Self {
        Self {blocks, entity}
    }
}
/*
impl Default for Chunk {
    fn default() -> Self {
        Self(Array::default((CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_LENGTH)))
    }
}
 */

// Resources
#[derive(Deref, DerefMut, Default)]
pub struct LoadedChunks(HashMap<IVec3, Chunk>);
impl LoadedChunks {
    pub fn index_block (index: IVec3) -> (IVec3, [usize; 3]) {
        (IVec3::new(index.x / CHUNK_WIDTH as i32, index.y / CHUNK_HEIGHT as i32, index.z / CHUNK_LENGTH as i32),
        [index.x as usize % CHUNK_WIDTH , index.y as usize % CHUNK_HEIGHT, index.z as usize & CHUNK_LENGTH])
    }

    pub fn get_block (&self, index: IVec3) -> Option<&Block> {
        let (chunk_index, block_index) = LoadedChunks::index_block(index);

        if let Some(chunk) = &self.get(&chunk_index) {
            Some(&chunk.blocks[block_index])
        }
        else {
            None
        }
    }

    pub fn get_block_mut (&mut self, index: IVec3) -> Option<&mut Block> {
        let (chunk_index, block_index) = LoadedChunks::index_block(index);

        if let Some(chunk) = self.get_mut(&chunk_index) {
            Some(&mut chunk.blocks[block_index])
        }
        else {
            None
        }
    }

    
    pub fn set_block_chunk (&mut self, index: IVec3, block: Block, commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>,) {
        let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
        material.perceptual_roughness = 0.9;

        if let Some(chunk) = self.get_mut(&index) {
            // Change the blocks of the chunk
            chunk.blocks = Array::from_elem((CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_LENGTH), block);
        }
        else {
            let blocks = Array::from_elem((CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_LENGTH), block);

           let chunk = commands
            .spawn_bundle(PbrBundle {
                mesh: generate_greedy_mesh(meshes, &blocks),
                material: materials.add(material),
                ..default()
            })
            .insert(Transform {
                translation: index.as_vec3(),
                //translation: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            })
            .insert(GlobalTransform::identity())
            .id();

            // Add chunk to loaded chunks
            self.entry(index).insert(Chunk::new(blocks, chunk));
        }
    }
     
}
 

// TODO: We should create a system which reads filenames to determine automatically what textures to use
// Or, we could just use a texture atlas and have preset coordinates.
// We could also stitch textures together into a single atlas on startup
#[derive(Deref, DerefMut, Default)]
pub struct BlockMaterials(EnumMap<BlockType, BlockMaterial>);

pub struct WithinBoxIterator {
    position: IVec3,
    min: IVec3,
    max: IVec3,
}
impl Iterator for WithinBoxIterator {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        self.position.x += 1;
        if self.position.x > self.max.x {
            self.position.x = self.min.x;
            self.position.y += 1;
        }
        if self.position.y > self.max.y {
            self.position.y = self.min.y;
            self.position.z += 1;
        }
        if self.position.z > self.max.z {
            return None
        }
        else {
            return Some(self.position)
        }
    }
}
impl WithinBoxIterator {
    pub fn new(min: IVec3, max: IVec3) -> WithinBoxIterator {
        WithinBoxIterator{position: min + IVec3::new(-1, 0, 0), min, max}
    }
}