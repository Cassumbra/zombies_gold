use std::ops::{Index, IndexMut};

use bevy::math::{Vec3A, const_ivec3};
use bevy::pbr::wireframe::WireframeConfig;
use bevy::render::mesh::{PrimitiveTopology, VertexAttributeValues, Indices};
use bevy::{prelude::*, utils::HashMap};
use enum_map::{EnumMap, Enum};
use ndarray::{Array3, Shape, Dim, Array};
use block_mesh::ndshape::{ConstShape, ConstShape3u32, ConstShape3usize};
use block_mesh::{greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};
use ndcopy::{copy3, fill3};

use crate::physics::AabbCollider;


// Consts
const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 16;
const CHUNK_LENGTH: usize = 16;

const BLOCK_SIDES: [IVec3; 6] = [const_ivec3!([-1, 0, 0]),
                                 const_ivec3!([1, 0, 0 ]),
                                 const_ivec3!([0, -1, 0]),
                                 const_ivec3!([0, 1, 0 ]),
                                 const_ivec3!([0, 0, -1]),
                                 const_ivec3!([0, 0, 1 ]),
                                ];

const CHUNK_SIDES: [usize; 6] = [0,
                                 CHUNK_WIDTH,
                                 0,
                                 CHUNK_HEIGHT,
                                 0,
                                 CHUNK_LENGTH,
                                ];

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
    let air = Block::new(BlockType::Air);

    for x in -2..2 {
        for z in -2..2 {
            for y in -2..2 {
                if y == -2 {
                    ev_set_block.send(SetBlockEvent { shape: SetBlockShape::Chunk(IVec3::new(x, y, z)), block: dirt })
                } else {
                    ev_set_block.send(SetBlockEvent { shape: SetBlockShape::Chunk(IVec3::new(x, y, z)), block: air })
                }
            }
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
                chunks.set_block_chunk(location, ev.block, &mut commands);
            }

            _ => {
                // TODO: add the other match arms lol
                todo!();
            }

        }


        
    }
}

pub fn lazy_mesher (
    chunks: Res<LoadedChunks>,

    mut ev_set_block_chunk: EventReader<SetBlockEvent>,

    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if ev_set_block_chunk.is_empty() {
        return;
    }

    let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
    material.perceptual_roughness = 0.9;

    let mut need_mesh = Vec::<IVec3>::new();

    for ev in ev_set_block_chunk.iter() {
        match ev.shape {
            SetBlockShape::Block(index) => {
                let (chunk_index, block_index) = LoadedChunks::index_block(index);

                add_no_dupe(&mut need_mesh, chunk_index);

                for i in 0..=2 {
                    if block_index[i] == CHUNK_SIDES[i] {
                        let modified_index = chunk_index + BLOCK_SIDES[i];
                        add_no_dupe(&mut need_mesh, modified_index);
                    }
                    else if block_index[i] == CHUNK_SIDES[i+1] {
                        let modified_index = chunk_index + BLOCK_SIDES[i+1];
                        add_no_dupe(&mut need_mesh, modified_index);
                    }
                }
            }
            SetBlockShape::Chunk(chunk_index) => {
                add_no_dupe(&mut need_mesh, chunk_index);

                for offset in BLOCK_SIDES {
                    let modified_index = chunk_index + offset;
                    add_no_dupe(&mut need_mesh, modified_index);
                }
            },
            
            _ => {
                todo!();
            }
        }
    }

    for location in need_mesh {
        if let Some(chunk) = chunks.get(&location) {
            let mesh = generate_greedy_mesh (&mut meshes, &chunks, location);

            commands.entity(chunk.entity)
                .insert(mesh)
                .insert(materials.add(material.clone()));
        }
    }
}

// Helper functions
fn add_no_dupe<T: PartialEq>(vec: &mut Vec<T>, val: T) {
    if !vec.contains(&val) {vec.push(val)};
}

// Yoinked from block-mesh examples with modifications cause I can't be assed.
fn generate_greedy_mesh(
    meshes: &mut Assets<Mesh>,
    chunks: &LoadedChunks,
    index: IVec3,
) -> Handle<Mesh> {

    type SampleShape = ConstShape3u32<{ CHUNK_WIDTH as u32 + 2 }, { CHUNK_HEIGHT as u32 + 2 }, { CHUNK_LENGTH as u32 + 2 }>;
    type ChunkShape = ConstShape3u32<{CHUNK_WIDTH as u32}, {CHUNK_HEIGHT as u32}, {CHUNK_LENGTH as u32}>;

    let mut samples = Array::from_elem((CHUNK_WIDTH + 2, CHUNK_HEIGHT + 2, CHUNK_LENGTH + 2), Block::new(BlockType::Air));
    let mut samples = samples.as_slice_mut().unwrap();

    copy3(
        [CHUNK_WIDTH as u32, CHUNK_HEIGHT as u32, CHUNK_LENGTH as u32],
        chunks[&index].blocks.as_slice().unwrap(),
        &ChunkShape {},
        [0, 0, 0],
        samples,
        &SampleShape {},
        [1, 1, 1],
    );

    let faces = [([1, CHUNK_HEIGHT as u32, CHUNK_LENGTH as u32], IVec3::new(-1, 0, 0), [CHUNK_WIDTH as u32 - 1, 0, 0] , [0, 1, 1])                          ,
                 ([1, CHUNK_HEIGHT as u32, CHUNK_LENGTH as u32], IVec3::new(1, 0, 0) , [0, 0, 0]                      , [CHUNK_WIDTH as u32 - 1 + 2, 1, 1]) ,
                 ([CHUNK_WIDTH as u32, 1, CHUNK_LENGTH as u32] , IVec3::new(0, -1, 0), [0, CHUNK_HEIGHT as u32 - 1, 0], [1, 0, 1])                          ,
                 ([CHUNK_WIDTH as u32, 1, CHUNK_LENGTH as u32] , IVec3::new(0, 1, 0) , [0, 0, 0]                      , [1, CHUNK_HEIGHT as u32 - 1 + 2, 1]),
                 ([CHUNK_WIDTH as u32, CHUNK_HEIGHT as u32, 1] , IVec3::new(0, 0, -1), [0, 0, CHUNK_LENGTH as u32 - 1], [1, 1, 0])                          ,
                 ([CHUNK_WIDTH as u32, CHUNK_HEIGHT as u32, 1] , IVec3::new(0, 0, 1) , [0, 0, 0]                      , [1, 1, CHUNK_LENGTH as u32 - 1 + 2]),
                ];

    for (shape, offset, src_start, dst_start) in faces {
        if let Some(chunk) = chunks.get(&(index + offset)) {
            copy3(
                shape,
                chunk.blocks.as_slice().unwrap(),
                &ChunkShape {},
                src_start,
                &mut samples,
                &SampleShape {},
                dst_start,
            );
        }
        else {

            let infinium = Block::new(BlockType::Infinium);

            fill3(
                shape,
                infinium,
                &mut samples,
                &SampleShape {},
                dst_start,
            );
        }
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new((CHUNK_WIDTH + 2) * (CHUNK_HEIGHT + 2) * (CHUNK_LENGTH + 2));
    greedy_quads(
        samples,
        &SampleShape {},
        [0; 3],
        [CHUNK_WIDTH as u32 + 2 - 1, CHUNK_HEIGHT as u32 + 2 - 1, CHUNK_LENGTH as u32 + 2 - 1],
        &faces,
        &mut buffer,
    );

    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                &quad,
            ));
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(tex_coords));
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    meshes.add(render_mesh)
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

    pub fn collidable (&self) -> bool {
        self.block_type.collidable()
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
    Infinium,
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
            BlockType::Infinium => {VoxelVisibility::Opaque}
            BlockType::Air => {VoxelVisibility::Empty}
            BlockType::Dirt => {VoxelVisibility::Opaque}

            _ => {VoxelVisibility::Opaque}
        }
    }
    pub fn collidable(&self) -> bool {
        match self {
            BlockType::Infinium => {true}
            BlockType::Air => {false}
            BlockType::Dirt => {true}

            _ => {true}
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

// Resources
#[derive(Deref, DerefMut, Default)]
pub struct LoadedChunks(HashMap<IVec3, Chunk>);
impl LoadedChunks {
    pub fn index_block (index: IVec3) -> (IVec3, [usize; 3]) {
        // block index is fine(?)
        // chunk index needs to be 1 lower for negative indexes
        let mut effective_index = index;

        if index.x < 0 {
            effective_index.x -= CHUNK_WIDTH as i32;
        }
        if index.y < 0 {
            effective_index.y -= CHUNK_HEIGHT as i32;
        }
        if index.z < 0 {
            effective_index.z -= CHUNK_LENGTH as i32;
        }

        (IVec3::new(effective_index.x / CHUNK_WIDTH as i32, effective_index.y / CHUNK_HEIGHT as i32, effective_index.z / CHUNK_LENGTH as i32),
        [index.x as usize % CHUNK_WIDTH, index.y as usize % CHUNK_HEIGHT, index.z as usize % CHUNK_LENGTH])
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

    
    pub fn set_block_chunk (&mut self, index: IVec3, block: Block, commands: &mut Commands) {
        if let Some(chunk) = self.get_mut(&index) {
            // Change the blocks of the chunk
            chunk.blocks = Array::from_elem((CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_LENGTH), block);
        }
        else {
            let blocks = Array::from_elem((CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_LENGTH), block);

            let chunk = commands
                .spawn()
                .insert(Transform {
                    translation: IVec3::new(CHUNK_WIDTH as i32 * index.x, CHUNK_HEIGHT as i32 * index.y, CHUNK_LENGTH as i32 * index.z).as_vec3(),
                    ..default()
                })
                .insert(GlobalTransform::identity())
                .insert(Visibility::default())
                .insert(ComputedVisibility::default())
                .id();

            // Add chunk to loaded chunks
            self.entry(index).insert(Chunk::new(blocks, chunk));
        }
    }

    
    pub fn aabb_collides(&self, direction: Vec3, aabb: AabbCollider) ->  Vec3 {
        // TODO: Return an infinity if the collider is squashed
        let mut need_check_x = Vec::<IVec3>::new();
        let mut need_check_y = Vec::<IVec3>::new();
        let mut need_check_z = Vec::<IVec3>::new();

        let min_x = aabb.min.x.round() as i32;
        let max_x = aabb.max.x.round() as i32;
        let min_y = aabb.min.y.round() as i32;
        let max_y = aabb.max.y.round() as i32;
        let min_z = aabb.min.z.round() as i32;
        let max_z = aabb.max.z.round() as i32;

        let mut normal_x = 0.0;
        let mut normal_y = 0.0;
        let mut normal_z = 0.0;

        if direction.x != 0.0 {
            // We get what our normal will be if we detect a collision later, and we set x to an appropriate value for detecting a collision
            // Note: I was adding 1 to max and subtracting it from min earlier. I think this is not necessary or good for me to do.
            //       Will see later. Leaving this note so I keep it in mind.
            let x = if direction.x > 0.0 {normal_x = 1.0; max_x} else {normal_x = -1.0; min_x};
            for y in (min_y)..=(max_y) {
                for z in (min_z)..=(max_z) {
                    need_check_x.push(IVec3::new(x, y, z))
                }
            }
        }

        if direction.y != 0.0 {
            let y = if direction.y > 0.0 {normal_y = 1.0; max_y} else {normal_y = -1.0; min_y};
            for x in (min_x)..=(max_x) {
                for z in (min_z)..=(max_z) {
                    need_check_y.push(IVec3::new(x, y, z))
                }
            }
        }

        if direction.z != 0.0 {
            let z = if direction.z > 0.0 {normal_z = 1.0; max_z} else {normal_z = -1.0; min_z};
            for x in (min_x)..=(max_x) {
                for y in (min_y)..=(max_y) {
                    need_check_z.push(IVec3::new(x, y, z))
                }
            }
        }


        let mut normal = Vec3::default();
        let blocksize = Vec3::new(1.0, 1.0, 1.0);

        //println!("PRINTED HERE FOR YOUR PERUSAL");
        for position in need_check_x {
            //println!("{:?}", position);
            let collidable = 
                if let Some(block) = self.get_block(position) {
                    block.collidable()
                }
                // Unloaded chunks act as collidable so the player doesn't go OOB
                else {
                    true
                }; 
            //println!("collidable: {}", collidable);

            if collidable {
                //println!("it collidable!");
                if aabb.compare_simple(AabbCollider::with_location(position.as_vec3(), blocksize)) {
                    normal.x = normal_x;
                    break
                }
            }
        }

        for position in need_check_y {
            let collidable = 
                if let Some(block) = self.get_block(position) {
                    //println!("there is a bloche here....");
                    block.collidable()
                }
                // Unloaded chunks act as collidable so the player doesn't go OOB
                else {
                    true
                };

            if collidable {
                if aabb.compare_simple(AabbCollider::with_location(position.as_vec3(), blocksize)) {
                    normal.y = normal_y;
                    break
                }
            }
        }

        for position in need_check_z {
            let collidable = 
                if let Some(block) = self.get_block(position) {
                    block.collidable()
                }
                // Unloaded chunks act as collidable so the player doesn't go OOB
                else {
                    true
                };

            if collidable {
                if aabb.compare_simple(AabbCollider::with_location(position.as_vec3(), blocksize)) {
                    normal.z = normal_z;
                    break
                }
            }
        }

        normal
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