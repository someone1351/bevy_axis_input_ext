
// use bevy::asset::saver::AssetSaver;
use bevy::prelude::*;

// use super::assets::*;
use super::resources::*;
// use super::values::*;
use super::utils::*;
// use std::fmt::Debug;

use std::{any::Any, hash::Hash, str::FromStr};
use bevy_axis_input as axis_input;

pub fn init<M:Send+Sync+Hash+Eq+FromStr+Any+Clone>( //+Debug
    // asset_server: Res<AssetServer>,
    mut input_config: ResMut<InputConfig<M>>,
    mut input_map : ResMut<axis_input::InputMap<M>>,
) {
    if let Some(conf)=load_input_conf::<M>(&input_config.default_file) {
        get_input_data(&conf, &mut input_config.default_data);
    }

    if let Some(conf)=load_input_conf::<M>(&input_config.user_file) {
        get_input_data(&conf, &mut input_config.user_data);
    }

    //
    input_config.last_user_data=input_config.user_data.clone();

    //
    input_map.bind_mode_excludes=input_config.get_excludes();
    input_map.bind_mode_includes=input_config.get_includes();
    input_map.mapping_repeats=input_config.get_mapping_repeats();

    if let Some((x,y))=input_config.get_bind_mode_deads() {
        input_map.bind_mode_start_dead=x;
        input_map.bind_mode_end_dead=y;
    }

    //
    apply_input_map(&mut input_config, &mut input_map.owner_bindings);
}

pub fn update<M:Send+Sync+Hash+Eq+Any+Clone+FromStr+ToString>( //+Debug
    // asset_server: Res<AssetServer>,
    mut input_config: ResMut<InputConfig<M>>,
    mut input_map : ResMut<axis_input::InputMap<M>>,
) {
    if input_config.do_save {
        input_config.do_save=false;

        if save_input_data(&input_config.user_data,&input_config.user_file) {
            println!("Input config file saved: {:?}.",input_config.user_file);
        }
    }

    //update input_map owner bindings
    if input_config.do_apply {
        input_config.do_apply=false;

        apply_input_map(&mut input_config, &mut input_map.owner_bindings);

        input_map.bindings_updated=true;
    }
}