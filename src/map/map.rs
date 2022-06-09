
use bevy::{
    pbr::wireframe::{WireframeConfig},
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::{PrimitiveTopology},
    },
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, VHACDParameters, RigidBody};
use block_mesh::ilattice::glam::Vec3A;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};




pub fn map_setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    wireframe_config.global = false;

    //let simple_sphere_handle = generate_simple_mesh(&mut meshes, |p| sphere(0.9, p));
    let greedy_sphere_handle = generate_greedy_mesh(&mut meshes, |p| sphere(0.9, p));

    /*
    if let Some(simple_sphere_mesh) = meshes.get(simple_sphere_handle.clone()) {
        spawn_pbr(
            &mut commands,
            &mut materials,
            simple_sphere_handle,
            simple_sphere_mesh,
            Transform::from_translation(Vec3::new(8.0, -16.0, -16.0)),
        );
    }
     */

    if let Some(greedy_sphere_mesh) = meshes.get(greedy_sphere_handle.clone()) {
        spawn_pbr(
            &mut commands,
            &mut materials,
            greedy_sphere_handle,
            greedy_sphere_mesh,
            Transform::from_translation(Vec3::new(-16.0, -16.0, 8.0)),
        );
    }
     

}

fn generate_simple_mesh(
    meshes: &mut Assets<Mesh>,
    sdf: impl Fn(Vec3A) -> BoolVoxel,
) -> Handle<Mesh> {
    type SampleShape = ConstShape3u32<34, 34, 34>;

    let mut samples = [EMPTY; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(32, SampleShape::delinearize(i));
        samples[i as usize] = sdf(p);
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = UnitQuadBuffer::new();
    visible_block_faces(
        &samples,
        &SampleShape {},
        [0; 3],
        [33; 3],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.num_quads() * 6;
    let num_vertices = buffer.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]));
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    meshes.add(render_mesh)
}

fn generate_greedy_mesh(
    meshes: &mut Assets<Mesh>,
    sdf: impl Fn(Vec3A) -> BoolVoxel,
) -> Handle<Mesh> {
    type SampleShape = ConstShape3u32<34, 34, 34>;

    let mut samples = [EMPTY; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(32, SampleShape::delinearize(i));
        samples[i as usize] = sdf(p);
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new(samples.len());
    greedy_quads(
        &samples,
        &SampleShape {},
        [0; 3],
        [33; 3],
        &faces,
        &mut buffer,
    );
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

    meshes.add(render_mesh)
}

fn spawn_pbr(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh_handle: Handle<Mesh>,
    mesh: &Mesh,
    transform: Transform,
) {
    let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
    material.perceptual_roughness = 0.9;

    if let Some(collider) = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh) { //ConvexDecomposition(VHACDParameters::default())
        commands.spawn_bundle(PbrBundle {
            mesh: mesh_handle,
            material: materials.add(material),
            ..Default::default()
        })
        .insert(transform)
        .insert(GlobalTransform::default())
        //.insert(CollisionShape::Cuboid {
        //    half_extends: Vec3::new(10.5, 0.1, 10.5),
        //    border_radius: None,
        //})
        .insert(collider)
        .insert(RigidBody::Fixed);
        //.insert(CollisionLayers::default());
    }


}

fn into_domain(array_dim: u32, [x, y, z]: [u32; 3]) -> Vec3A {
    (2.0 / array_dim as f32) * Vec3A::new(x as f32, y as f32, z as f32) - 1.0
}

fn sphere(radius: f32, p: Vec3A) -> BoolVoxel {
    BoolVoxel(p.length() < radius)
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);

impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}
