use bevy::{prelude::*, input::mouse::MouseMotion};
use heron::prelude::*;
use leafwing_input_manager::{Actionlike, prelude::ActionState};

const SPEED: f32 = 8.;
const ACCELERATION: f32 = 2.;

// Systems
pub fn process_actions(
    mut windows: ResMut<Windows>,
    time: Res<Time>,

    mut motion_evr: EventReader<MouseMotion>,

    mut camera_query: Query<(&mut Transform), (With<Camera>, With<Parent>)>,
    mut query: Query<(&Children, &ActionState<Action>, &mut Velocity, &mut Transform), Without<Camera>>
) {
    let sensitivity_mult = 0.005;
    let window = windows.get_primary_mut().unwrap();

    for (cameras, action_state, mut velocity, mut transform) in query.iter_mut() {
        if window.cursor_locked() && window.is_focused() {
            for ev in motion_evr.iter() {
                transform.rotate(Quat::from_rotation_y(-ev.delta.x * sensitivity_mult));
                for camera in cameras.iter() {
                    if let Ok(mut camera_transform) = camera_query.get_mut(*camera) {
                        camera_transform.rotate(Quat::from_rotation_x(-ev.delta.y * sensitivity_mult));
                        
                        //camera_transform.rotation.x - 
                        let lock_val = camera_transform.rotation.x.clamp(-0.523599, 0.523599) - camera_transform.rotation.x;
                        camera_transform.rotate(Quat::from_rotation_x(lock_val));
                    }
                }
            }
        }

        
        if action_state.just_pressed(Action::Jump) {
            println!("boioing");
            velocity.linear += Vec3::new(0., 5., 0.); 
        }

        let mut direction = Vec3::default();
        if action_state.pressed(Action::WalkForward) {
            direction += -transform.local_z();
        }
        else if action_state.pressed(Action::WalkBackward) {
            direction += transform.local_z();
        }

        if action_state.pressed(Action::StrafeLeft) {
            direction += -transform.local_x();
        }
        else if action_state.pressed(Action::StrafeRight) {
            direction += transform.local_x();
        }
        
        if direction != Vec3::default() {
            let mut velocity_add = ((direction.normalize_or_zero()*SPEED).lerp(velocity.linear, 0.0) - velocity.linear) * time.delta_seconds();
            velocity_add.y = 0.;
            velocity.linear += velocity_add;
        } else {
            let mut velocity_add = (velocity.linear.lerp(Vec3::default(), 1.0) - velocity.linear) * time.delta_seconds();
            velocity_add.y = 0.;
            velocity.linear += velocity_add;
        }
    }
}

// Data
#[derive(Hash, PartialEq, Eq, Clone, Actionlike)]
pub enum Action {
    WalkForward,
    WalkBackward,
    StrafeLeft,
    StrafeRight,
    Jump,
    Crouch,
    //LookUp,
    //LookDown,
    //LookLeft,
    //LookRight,
}