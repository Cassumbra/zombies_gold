use bevy::{prelude::*, math::Vec3A};
use iyes_loopless::state::NextState;
use leafwing_input_manager::prelude::*;

use crate::{actions::Action, player::Player, GameState, physics::{AabbCollider, Velocity, Falls}};

//use super::{GameState, TextureAssets};

const PLAYER_HEIGHT: f32 = 0.4;

// Systems
pub fn spawn_actors (
    mut commands: Commands,
) {
    let spawn_pos = Vec3::new(0.0, 0.0, 0.0);

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
        .insert(Falls)
        .insert(AabbCollider::new(Vec3A::new(0.4, 1.8, 0.4)))
        .insert(Velocity(Vec3::new(0.0, 0.0, 0.0)))
        .insert(Transform {
            translation: spawn_pos,
            ..default()
        })
        .insert(GlobalTransform::identity())
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


    commands.insert_resource(NextState(GameState::Playing));
}