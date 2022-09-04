use bevy::prelude::*;

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
	fn build(&self, app: &mut App) {
		// app.insert_resource(MainMenuSkillSelection(1));
		// app.add_system_set(
		// 	SystemSet::on_enter(GameState::LevelSelectionMenu)
		// 		// .with_system(enter)
		// 		.with_system(spawn_background)
		// 		.with_system(spawn_levels)
		// );
		// app.add_system_set(
		// 	SystemSet::on_update(GameState::LevelSelectionMenu)
		// 		// .with_system(update)
		// 		.with_system(button_highlight_system)
		// 		.with_system(button_system)
		// );
		// app.add_system_set(
		//     SystemSet::on_exit(GameState::LevelSelectionMenu)
		//         .with_system(exit),
		// );
	}
}

#[derive(Component)]
struct InGameComponent; // Marker component so it can be despawned.
