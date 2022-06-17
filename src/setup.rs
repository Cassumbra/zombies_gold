use bevy::{prelude::*};
use bevy_rapier3d::prelude::{RigidBody, Velocity, LockedAxes, Collider};
use iyes_loopless::state::NextState;
use leafwing_input_manager::prelude::*;

use crate::{actions::Action, player::Player, GameState};

//use super::{GameState, TextureAssets};

const PLAYER_HEIGHT: f32 = 0.4;

// Systems
pub fn spawn_actors (
    mut commands: Commands,
) {
    println!("Spawning actors");
    let spawn_pos = Vec3::new(8.0, 0.0, 0.0);

    // Player
    commands
        .spawn_bundle(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([(Action::Jump, KeyCode::Space),
                                      (Action::Crouch, KeyCode::LControl),
                                      (Action::StrafeRight, KeyCode::D),
                                      (Action::StrafeLeft, KeyCode::A),
                                      (Action::WalkForward, KeyCode::W),
                                      (Action::WalkBackward, KeyCode::S),
                                     ])
        })
        .insert(Player)
        .insert(Transform {
            translation: spawn_pos,
            ..default()
        })
        
        .insert(GlobalTransform::identity())
        .insert(Collider::capsule(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, PLAYER_HEIGHT, 0.0), 0.2))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        //.insert(CollisionShape::Capsule {
        //    radius: 0.2,
        //    half_segment: PLAYER_HEIGHT / 2.0,
        //})
        // Camera
        .with_children(|c| {
            c.spawn_bundle(PerspectiveCameraBundle::new_3d())
                .insert( Transform {
                    translation: Vec3::new(0.0, PLAYER_HEIGHT - (PLAYER_HEIGHT / 4.0), 0.0),
                    ..default()
                });
        })
        // light
        .with_children(|c| {
            c.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    intensity: 200.0,
                    //range: 80.0,
                    //radius: 80.0,
                    ..default()
                },
                ..default()
            });
        });

    println!("wauw");

    commands.insert_resource(NextState(GameState::Playing));
}