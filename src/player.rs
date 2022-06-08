use bevy::{prelude::*, app::AppExit};

// Components
#[derive(Component)]
pub struct Player;

// Systems
pub fn meta_input (
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,

    mut ev_exit: EventWriter<AppExit>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Tab) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }

    if key.just_pressed(KeyCode::Escape) {
        ev_exit.send(AppExit);
    }
}