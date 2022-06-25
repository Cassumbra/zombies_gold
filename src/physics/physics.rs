use bevy::{prelude::*, math::Vec3A};
use bevy_inspector_egui::Inspectable;

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
    mut velocity_query: Query<(&Velocity, &mut Transform)>,

    time: Res<Time>,
) {
    for (velocity, mut transform) in  velocity_query.iter_mut() {
        transform.translation += **velocity * time.delta_seconds();
    }
}

pub fn apply_gravity(
    mut velocity_query: Query<(&mut Velocity), With<Falls>>,

    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    // TODO: Per every second, we should apply one (1) gravity.
    for mut velocity in  velocity_query.iter_mut() {
        **velocity += **gravity * time.delta_seconds();
    }
}

// Components
#[derive(Component)]
pub struct AabbCollider {
    pub min: Vec3,
    pub max: Vec3,
    // Gets updated in collision detection and used in collision response
    // Ex: colliding exclusively with floor would be 0.0, 1.0, 0.0
    pub normal: Vec3,
}
impl AabbCollider {
    pub fn new(size: Vec3A) -> Self {
        Self {min: Vec3::from(size / -2.0), max: Vec3::from(size / 2.0), normal: Vec3::new(0.0, 0.0, 0.0)}
    }
}

#[derive(Component, Deref, DerefMut, Debug, Reflect, Inspectable)]
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