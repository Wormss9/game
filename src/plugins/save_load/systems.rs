use std::fs;
use std::fs::{DirEntry, File};
use std::io::Write;

use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;

use filetime::FileTime;
use regex::Regex;
use crate::plugins::components::Save;


const SAVE_FILE_AMOUNT: u32 = 5;

pub fn quick_save_game(
    keyboard_input: Res<Input<KeyCode>>,
    world: &World,
) {
    if keyboard_input.just_released(KeyCode::F5) {
        let app_type_registry = world.resource::<AppTypeRegistry>();

        let scene = DynamicScene::from_world(world, app_type_registry);

        let serialized_scene = scene.serialize_ron(app_type_registry as &TypeRegistryArc).unwrap();
        let last_file = get_last_save();

        let file_name = match last_file {
            None => { "0.scn.ron".to_string() }
            Some(last_file) => { last_file.file_name().into_string().unwrap() }
        };

        let mut number = file_name.split_at(file_name.len() - 8).0.parse::<i32>().unwrap() + 1;
        if number > SAVE_FILE_AMOUNT as i32 {
            number = 1
        }

        let mut file = File::create("assets/saves/".to_string() + &number.to_string() + ".scn.ron").unwrap();
        file.write_all((serialized_scene).as_bytes()).unwrap();

        info!("Saved")
    }
}

pub fn quick_load_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    query: Query<Entity, &Save>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_released(KeyCode::F9) {
        let last_file = get_last_save();

        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        if last_file.is_none() {
            warn!("No save file found");
            return;
        }

        let x = last_file.unwrap().path();

        let path = x.to_str().unwrap().split_at(7).1;

        commands.spawn(DynamicSceneBundle {
            // Scenes are loaded just like any other asset.
            scene: asset_server.load(path),
            ..default()
        });
    }
}

fn get_last_save() -> Option<DirEntry> {
    let regex = Regex::new(r"^\d*\.scn\.ron$").unwrap();
    fs::create_dir_all("assets/saves").expect("Failed to create dir");
    fs::read_dir("assets/saves").expect("Failed to read dir").filter(|result| match result {
        Ok(_) => true,
        Err(_) => false
    })
        .map(|x| x.unwrap())
        .filter(|file| regex.is_match(file.file_name().to_str().expect("Bad filename")))
        .max_by_key(|save_file| FileTime::from_last_modification_time(&save_file.metadata().unwrap()))
}