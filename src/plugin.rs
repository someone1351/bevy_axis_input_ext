
use std::path::PathBuf;
use std::{any::Any, str::FromStr,fmt::Debug,hash::Hash};

use bevy::prelude::*;

use super::resources::*;
use super::systems::*;
// use super::assets::*;

// #[derive(Default)]
pub struct InputConfigPlugin<M:Sync+Send+Any+Clone+Eq+Hash+Debug+FromStr+ToString>{
    pub phantom_data:std::marker::PhantomData<M>,
    pub user_file_path : String,
    pub default_file_path : String,
}

impl<M:Sync+Send+Any+Clone+Eq+Hash+Debug+FromStr+ToString> Default for InputConfigPlugin<M> {
    fn default() -> Self {
        Self {
            phantom_data: Default::default(),
            user_file_path: "config".to_string(),
            default_file_path: "config".to_string(),
        }
    }
}

impl<M:Sync+Send+Any+Clone+Eq+Hash+Debug+FromStr+ToString> bevy::app::Plugin for InputConfigPlugin<M> {
    fn build(&self, app: &mut bevy::app::App) {
        let mut input_config=app.world_mut().get_resource_or_init::<InputConfig<M>>();

        input_config.default_file=PathBuf::from(&self.default_file_path);
        input_config.default_file.push("default.input_conf");

        input_config.user_file=PathBuf::from(&self.user_file_path);
        input_config.user_file.push("user.input_conf");

        app
            // .init_asset::<InputAsset>()
            // .init_asset_loader::<InputAssetLoader<M>>()

            // .init_resource::<InputConfig<M>>()
            .add_systems(Startup, ( init::<M>, ))
            .add_systems(Update,(
                update::<M>,
                // on_modified::<M>, load::<M>,
            ).chain())
        ;
    }
}

