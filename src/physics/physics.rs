use bevy::{prelude::*, math::Vec3A, asset::LoadContext};
use bevy_inspector_egui::Inspectable;

use crate::map::BLOCK_SIZE;
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
    mut velocity_query: Query<(&mut Velocity, &mut Transform, Option<&mut AabbCollider>)>,

    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
) {
    for (mut velocity, mut transform, mut opt_aabb) in velocity_query.iter_mut() {

        if **velocity == Vec3::default() {
            println!("dude");
            continue
        }
        // TODO: Check whether to use air resistance or ground resistance and use it.
        // TODO: Check if the direction we're moving has anything collidable and cancel velocity if it does
        if let Some(mut aabb) = opt_aabb {
            // TODO: Send in velocity * delta time to the collision stuff
            // TODO: Get list of possible voxels we could collide with sorted by distance squared
            // TODO: HEY WAIT A MINUTE: Isn't that the same as creating a broadphase thingy???? kinda??? do we even need that???? me dunno...

            let mut remaining_time = 1.0;

            while remaining_time > 0.0 {
                //println!("position: {}", transform.translation);
                println!("velocity: {}", **velocity);
                let mut modified_velocity = **velocity * time.delta_seconds() * remaining_time;
                let positions = loaded_chunks.broadphase(&aabb, &transform.translation, &modified_velocity);
                if positions.len() == 0 {
                    break
                }
                //let mut collision_time = 0.0;
                //let mut modified_velocity = **velocity;

                for (position, _distance) in positions {
                    remaining_time = aabb.compare_swept(transform.translation, modified_velocity, BLOCK_SIZE, position);
                    println!("remaining time: {}", remaining_time);
                    // 1 means no collision, anything lower is a collision.
                    if remaining_time < 1.0 {
                        break
                    }
                }

                if remaining_time < 1.0 {
                    (remaining_time, modified_velocity) = aabb.response_slide(&mut transform.translation, &modified_velocity, remaining_time);
                    //println!("modified velocity be: {}", modified_velocity);
                    //println!("time is: {}", remaining_time);
                    //println!("UH OH: {}", 1.0 / remaining_time);
                    if remaining_time > 0.0 {
                        **velocity = modified_velocity * (1.0 / time.delta_seconds()) * (1.0 / remaining_time);
                    } else {
                        **velocity = Vec3::default();
                    }
                    
                }
            }
        }
        //else {
            //println!("VELOCITY IS: {}", **velocity);
            transform.translation += **velocity * time.delta_seconds();
        //}
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
#[derive(Component, Clone, Copy, Debug, Reflect, Inspectable)]
pub struct AabbCollider {
    /// Width, height, length.
    pub extents: Vec3,
    //pub min: Vec3,
    //pub max: Vec3,
    // Gets updated in collision detection and used in collision response
    // Ex: colliding exclusively with floor would be 0.0, 1.0, 0.0
    pub normal: Vec3,
}
impl Default for AabbCollider {
    fn default() -> Self {
        Self { extents: Vec3::new(1.0, 1.0, 1.0), normal: Vec3::default() }
        //Self { min: Vec3::new(-0.5, -0.5, -0.5), max: Vec3::new(0.5, 0.5, 0.5), normal: Vec3::new(0.0, 0.0, 0.0) }
    }
}
impl AabbCollider {
    pub fn new(extents: Vec3) -> Self {
        Self { extents, ..default()}
    }

    pub fn broadphase(&self, position: &Vec3, velocity: &Vec3) -> (Vec3, Vec3) {
        let mut location = Vec3::default();
        let mut extents = Vec3::default();
        
        location.x = if velocity.x > 0.0 {position.x} else {position.x + velocity.x};
        location.y = if velocity.y > 0.0 {position.y} else {position.y + velocity.y};
        location.z = if velocity.z > 0.0 {position.z} else {position.z + velocity.z};
        extents.x = if velocity.x > 0.0 {self.extents.x + velocity.x} else {self.extents.x - velocity.x};
        extents.y = if velocity.y > 0.0 {self.extents.y + velocity.y} else {self.extents.y - velocity.y};
        extents.z = if velocity.z > 0.0 {self.extents.z + velocity.z} else {self.extents.z - velocity.z};

        (location, extents)
    }

    pub fn compare_simple (self, position: Vec3, collider_extents: Vec3, collider_position: Vec3) -> bool {
        // TODO: Should these be <= and >=??
        // TODO: Is this inverse check method more effecient than just checking with &&?
        !(position.x + self.extents.x < collider_position.x || position.x > collider_position.x + collider_extents.x ||
          position.y + self.extents.y < collider_position.y || position.y > collider_position.y + collider_extents.y ||
          position.z + self.extents.z < collider_position.z || position.z > collider_position.z + collider_extents.z
         )
    }

    // Returns collision time. Changes normals of collider.
    pub fn compare_swept (&mut self, position: Vec3, velocity: Vec3, collider_extents: Vec3, collider_position: Vec3) -> f32 {
        let mut inverse_entry = Vec3::default();
        let mut inverse_exit = Vec3::default();

        if velocity.x > 0.0 {
            inverse_entry.x = collider_position.x - (position.x + self.extents.x);
            inverse_exit.x = (collider_position.x + collider_extents.x) - position.x;
        }
        else {
            inverse_entry.x = (collider_position.x + collider_extents.x) - position.x;
            inverse_exit.x = collider_position.x - (position.x + self.extents.x);
        }

        if velocity.y > 0.0 {
            inverse_entry.y = collider_position.y - (position.y + self.extents.y);
            inverse_exit.y = (collider_position.y + collider_extents.y) - position.y;
        }
        else {
            inverse_entry.y = (collider_position.y + collider_extents.y) - position.y;
            inverse_exit.y = collider_position.y - (position.y + self.extents.y);
        }

        if velocity.z > 0.0 {
            inverse_entry.z = collider_position.z - (position.z + self.extents.z);
            inverse_exit.z = (collider_position.z + collider_extents.z) - position.z;
        }
        else {
            inverse_entry.z = (collider_position.z + collider_extents.z) - position.z;
            inverse_exit.z = collider_position.z - (position.z + self.extents.z);
        }

        let mut entry = Vec3::default();
        let mut exit = Vec3::default();

        if velocity.x == 0.0 {
            entry.x = f32::NEG_INFINITY;
            exit.x = f32::INFINITY;
        }
        else {
            entry.x = inverse_entry.x / velocity.x;
            exit.x = inverse_exit.x / velocity.x;
        }

        if velocity.y == 0.0 {
            entry.y = f32::NEG_INFINITY;
            exit.y = f32::INFINITY;
        }
        else {
            entry.y = inverse_entry.y / velocity.y;
            exit.y = inverse_exit.y / velocity.y;
        }

        if velocity.z == 0.0 {
            entry.z = f32::NEG_INFINITY;
            exit.z = f32::INFINITY;
        }
        else {
            entry.z = inverse_entry.z / velocity.z;
            exit.z = inverse_exit.z / velocity.z;
        }

        let entry_time = entry.max_element();
        let exit_time = exit.min_element();

        // No collision
        if entry_time > exit_time || entry.x > 1.0 || entry.y > 1.0 || entry.z > 1.0 ||(entry.x < 0.0 && entry.y < 0.0 && entry.z < 0.0) {
            self.normal = Vec3::default();

            1.0
        }
        //Collision 
        else  {
            if (entry.x > entry.y && entry.x > entry.z) { 
                if (inverse_entry.x < 0.0) { 
                    self.normal.x = 1.0; 
                    self.normal.y = 0.0;
                    self.normal.z = 0.0; 
                } 
                else { 
                    self.normal.x = -1.0; 
                    self.normal.y = 0.0;
                    self.normal.z = 0.0; 
                } 
            } 
            else if (entry.y > entry.z) { 
                if (inverse_entry.y < 0.0) { 
                    self.normal.x = 0.0; 
                    self.normal.y = 1.0;
                    self.normal.z = 0.0; 
                } 
                else { 
                    self.normal.x = 0.0; 
                    self.normal.y = -1.0;
                    self.normal.z = 0.0; 
                } 
            }
            else{ 
                if (inverse_entry.z < 0.0) { 
                    self.normal.x = 0.0; 
                    self.normal.y = 0.0;
                    self.normal.z = 1.0; 
                } 
                else { 
                    self.normal.x = 0.0; 
                    self.normal.y = 0.0;
                    self.normal.z = -1.0; 
                } 
            }

            entry_time
        }
    }

    // Returns remaining time. Changes position.
    pub fn response_base (self, position: &mut Vec3, velocity: Vec3, collision_time: f32) -> f32 {
        *position += velocity * collision_time;
        1.0 - collision_time
    }

    // Returns remaining time. Changes position. Changes velocity.
    pub fn response_deflect (self, position: &mut Vec3, velocity: &mut Vec3, collision_time: f32) -> f32 {
        let remaining_time = self.response_base(position, *velocity, collision_time);

        *velocity *= remaining_time;
        if self.normal.x.abs() > 0.0001 {velocity.x = -velocity.x}
        if self.normal.y.abs() > 0.0001 {velocity.y = -velocity.y}
        if self.normal.z.abs() > 0.0001 {velocity.z = -velocity.z}

        remaining_time
    }

    // Returns remaining time. Changes position. Changes velocity.
    pub fn response_push (self, position: &mut Vec3, velocity: &mut Vec3, collision_time: f32) -> f32 {
        let remaining_time = self.response_base(position, *velocity, collision_time);

        let magnitude = velocity.length() * remaining_time;
        let mut dot = velocity.dot(self.normal);

        if dot > 0.0 {dot = 1.0}
        else if dot < 0.0 {dot = -1.0}

        *velocity = dot * self.normal * magnitude;

        remaining_time
    }

    // Returns remaining time. Returns velocity. Changes position.
    pub fn response_slide (self, position: &mut Vec3, velocity: &Vec3, collision_time: f32) -> (f32, Vec3) {
        let remaining_time = self.response_base(position, *velocity, collision_time);
    
        let dot = velocity.dot(self.normal);

        (remaining_time, dot * self.normal)
    }

    /*
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
     */
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