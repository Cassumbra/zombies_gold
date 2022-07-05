use bevy::{prelude::*, math::Vec3A, asset::LoadContext};
use bevy_inspector_egui::Inspectable;

use crate::map::{LoadedChunks, BlockType, Block};

// Plugin
#[derive(Default)]
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
         .init_resource::<Gravity>();

    }
}

// Resources
#[derive(Deref, DerefMut)]
pub struct Gravity (Vec3);
impl Default for Gravity {
    fn default() -> Self {
        Self(Vec3::new(0.0, -9.807, 0.0))
    }
}

// Systems
pub fn apply_velocity (
    mut velocity_query: Query<(&mut Velocity, &mut Transform, Option<&AabbCollider>)>,

    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
) {
    for (mut velocity, mut transform, opt_aabb) in  velocity_query.iter_mut() {
        // TODO: Check whether to use air resistance or ground resistance and use it.
        // TODO: Check if the direction we're moving has anything collidable and cancel velocity if it does
        if let Some(aabb) = opt_aabb {
            let velocities = velocity.to_array();
            // actually i'm not sure if we need to map these? it might be fine to keep them as references
            let mut velocities: Vec<(usize, f32)> = velocities.iter().enumerate().map(|(axis, mag)| (axis, *mag)).collect();
            velocities.sort_by(|(_axis0, mag0), (_axis1, mag1)| mag0.abs().partial_cmp(&mag1.abs()).unwrap());
            

            let half_extents = aabb.get_half_extents();
            for (axis, mag) in velocities {
                let modified_aabb = AabbCollider::add_location(transform.translation, aabb); //+ **velocity
                let (normal, collision) = loaded_chunks.aabb_collides_simple(axis, **velocity, modified_aabb);
                //println!("axis: {}, normal: {}", axis, normal);
                println!("location: {}", transform.translation);
                println!("aabb: {:?}", modified_aabb);

                if normal > 0.0 && mag > 0.0 {
                    velocity[axis] = 0.0;
                    transform.translation[axis] = collision[axis] - 0.5 - half_extents[axis];
                }
                else if normal < 0.0 && mag < 0.0 {
                    velocity[axis] = 0.0;
                    transform.translation[axis] = collision[axis] + 0.5 + half_extents[axis];
                }
                else {
                    transform.translation[axis] += velocity[axis] * time.delta_seconds();
                }
            }
        }
        else {
            transform.translation += **velocity * time.delta_seconds();
        }
        
    }
}

pub fn apply_gravity (
    mut velocity_query: Query<(&mut Velocity), With<Falls>>,

    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    for mut velocity in  velocity_query.iter_mut() {
        **velocity += **gravity * time.delta_seconds();
    }
}

// Components
#[derive(Component, Clone, Copy, Debug)]
pub struct AabbCollider {
    pub min: Vec3,
    pub max: Vec3,
    // Gets updated in collision detection and used in collision response
    // Ex: colliding exclusively with floor would be 0.0, 1.0, 0.0
    pub normal: Vec3,
}
impl Default for AabbCollider {
    fn default() -> Self {
        Self { min: Vec3::new(-0.5, -0.5, -0.5), max: Vec3::new(0.5, 0.5, 0.5), normal: Vec3::new(0.0, 0.0, 0.0) }
    }
}
impl AabbCollider {
    pub fn new(size: Vec3A) -> Self {
        Self {min: Vec3::from(size / -2.0), max: Vec3::from(size / 2.0), ..default()}
    }
    pub fn with_location(location: Vec3, size: Vec3) -> Self {
        Self {min: location + (size * -0.5), max: location + (size * 0.5), ..default()}
    }
    pub fn add_location(location: Vec3, aabb: &AabbCollider) -> Self {
        Self {min: location + aabb.min, max: location + aabb.max, ..default()}
    }

    pub fn get_center(self) -> Vec3 {
        (self.max + self.min) * 0.5
    }
    pub fn get_extents(self) -> Vec3 {
        self.max - self.min
    }
    pub fn get_half_extents(self) -> Vec3 {
        self.get_extents() * 0.5
    }

    /// Compares this AABB in relation to another. Returns a normal.
    pub fn compare(self, collider: AabbCollider) -> Vec3 {
        // TODO: Add special case for if one collider is contained entirely within another?
        if self.min.x <= collider.max.x && self.max.x >= collider.min.x &&
           self.min.y <= collider.max.y && self.max.y >= collider.min.y &&
           self.min.z <= collider.max.z && self.max.z >= collider.min.z 
        {
            let self_center = self.get_center();
            let collider_center = collider.get_center();

            let lengths = collider_center - self_center;

            let lengths_abs = lengths.abs();

            if lengths_abs.x > lengths_abs.y && lengths_abs.x > lengths_abs.z {
                if lengths.x < 0.0 {
                    Vec3::new(-1.0, 0.0, 0.0)
                }
                else {
                    Vec3::new(1.0, 0.0, 0.0)
                }
            }
            else if lengths_abs.y > lengths_abs.z {
                if lengths_abs.y < 0.0 {
                    Vec3::new(0.0, -1.0, 0.0)
                }
                else {
                    Vec3::new(0.0, 1.0, 0.0)
                }
            }
            else {
                if lengths.z < 0.0 {
                    Vec3::new(0.0, 0.0, -1.0)
                }
                else {
                    Vec3::new(0.0, 0.0, 1.0)
                }
            }
        }
        else {
            Vec3::default()
        }
    }

    /// Compares this AABB in relation to another. Returns a normal using the direction.
    /// Requires less operations.
    pub fn compare_efficient(self, collider: AabbCollider, direction: Vec3) -> Vec3 {
        // TODO: Add special case for if one collider is contained entirely within another?
        if self.min.x <= collider.max.x && self.max.x >= collider.min.x &&
           self.min.y <= collider.max.y && self.max.y >= collider.min.y &&
           self.min.z <= collider.max.z && self.max.z >= collider.min.z 
        {

            let direction_abs = direction.abs();

            if direction_abs.x > direction_abs.y && direction_abs.x > direction_abs.z {
                if direction.x < 0.0 {
                    Vec3::new(-1.0, 0.0, 0.0)
                }
                else {
                    Vec3::new(1.0, 0.0, 0.0)
                }
            }
            else if direction_abs.y > direction_abs.z {
                if direction_abs.y < 0.0 {
                    Vec3::new(0.0, -1.0, 0.0)
                }
                else {
                    Vec3::new(0.0, 1.0, 0.0)
                }
            }
            else {
                if direction.z < 0.0 {
                    Vec3::new(0.0, 0.0, -1.0)
                }
                else {
                    Vec3::new(0.0, 0.0, 1.0)
                }
            }
        }
        else {
            Vec3::default()
        }
    }

    /// Compares this AABB in relation to another. Returns a bool.
    pub fn compare_simple(self, collider: AabbCollider) -> bool {
        self.min.x <= collider.max.x && self.max.x >= collider.min.x &&
        self.min.y <= collider.max.y && self.max.y >= collider.min.y &&
        self.min.z <= collider.max.z && self.max.z >= collider.min.z
    }
}

#[derive(Copy, Clone, Component, Deref, DerefMut, Debug, Reflect, Inspectable)]
pub struct Velocity (pub Vec3);

#[derive(Component)]
pub struct Falls;


#[derive(Component)]
pub struct AirResistance(Vec3);
impl Default for AirResistance {
    fn default() -> Self {
        Self(Vec3::new(0.0, 0.2, 0.0))
    }
}

#[derive(Component)]
pub struct GroundResistance(Vec3);
impl Default for GroundResistance {
    fn default() -> Self {
        Self(Vec3::new(1.0, 1.0, 1.0))
    }
}