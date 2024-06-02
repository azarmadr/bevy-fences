use bevy::prelude::*;

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod game;
mod menu;

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fences".to_string(),
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                resolution: (480., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, camera_setup)
        .add_plugins(menu::MenuPlugin)
        .add_plugins(game::BoardPlugin);

    #[cfg(debug_assertions)]
    {
        app.add_plugins(WorldInspectorPlugin::new());
    }
    app.run();
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
