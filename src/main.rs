// Uhm, actually, its 3D ZOMBIES GOLD ????~

use bevy::{
    pbr::wireframe::WireframePlugin,
    prelude::*,
};
use bevy_rapier3d::{plugin::{RapierPhysicsPlugin, NoUserData}, prelude::RapierDebugRenderPlugin};
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};
use leafwing_input_manager::plugin::InputManagerPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;


#[path = "map/map.rs"]
pub mod map;

pub mod actions;

pub mod player;

pub mod setup;


fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor{
            title: "3D ZOMBIES GOLD".to_string(),
            resizable: false,
            ..Default::default()}
        )

        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InputManagerPlugin::<actions::Action>::default())
        .add_plugin(WireframePlugin)
        .add_plugin(WorldInspectorPlugin::new())

        .add_plugin(map::MapPlugin)

        //.add_loopless_state(GameState::Loading)

        .add_loopless_state(GameState::Playing)

        .add_startup_system(setup::spawn_actors)
        .add_startup_system(map::map_setup)

        .add_system(map::set_block_chunk)

        .add_system(map::lazy_mesher.after(map::set_block_chunk))

        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .with_system(actions::process_actions)
                .with_system(player::meta_input)
                .into()
        )

        
        .run();
}

// Data
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Loading,
    StartMapGen, MapGen, SpawnActors,
    Playing,
}