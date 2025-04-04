
// use bevy::asset::saver::AssetSaver;
use bevy::prelude::*;

// use super::assets::*;
use super::resources::*;
// use super::values::*;
use super::utils::*;
use std::cmp::Ordering;
// use std::fmt::Debug;


use std::collections::HashMap;
use std::{any::Any, hash::Hash, str::FromStr};
use bevy_axis_input as axis_input;

pub fn init<M:Send+Sync+Hash+PartialEq+Eq+FromStr+Any+Clone>( //+Debug
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

    input_map.bind_mode_excludes=input_config.get_excludes();
    input_map.bind_mode_includes=input_config.get_includes();
    input_map.mapping_repeats=input_config.get_mapping_repeats();

    if let Some((x,y))=input_config.get_bind_mode_deads() {
        input_map.bind_mode_start_dead=x;
        input_map.bind_mode_end_dead=y;
    }

    // input_map.owner_bindings.clear();
    // let owner=input_map.owner_bindings.entry(0).or_default();

}

pub fn update<M:Send+Sync+Hash+PartialEq+Eq+FromStr+Any+Clone+ToString>( //+Debug
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
    if input_config.bindings_updated {
        input_config.bindings_updated=false;
        input_map.owner_bindings.clear();

        for (&owner,profiles) in input_config.owner_profiles.iter() {
            let bindings_out: &mut HashMap<(M, Vec<axis_input::Binding>), (f32, f32, f32)>=input_map.owner_bindings.entry(owner).or_default();

            //
            let mut profiles = profiles.clone();

            for profile in profiles.clone() {
                for i in (1 .. profile.len()).rev() {
                    profiles.insert(profile.get(0..i).unwrap().to_vec());
                }
            }

            //
            let mut profiles = profiles.iter().map(|x|x.clone()).collect::<Vec<_>>();

            profiles.sort_by(|x,y|{
                match y.len().cmp(&x.len()) {
                    Ordering::Equal => {
                        //compare names
                        for i in (0.. x.len()).rev() {
                            match x[i].cmp(&y[i]) {
                                Ordering::Equal => {}
                                y => {return y;}
                            }
                        }

                        //never actually reached
                        Ordering::Equal
                    },
                    x=>x
                }
            });

            //
            for profile in profiles {
                // println!("= profile {profile:?}");

                let mapping_bindings=input_config.get_bindings(&profile);

                for (mapping,binding_group_scales) in mapping_bindings {
                    for (binding_group,scale) in binding_group_scales {
                        let k =(mapping.clone(),binding_group.iter().map(|x|x.clone()).collect::<Vec<_>>());
                        // println!("bg {binding_group:?}");

                        if !bindings_out.contains_key(&k) {
                            let mut scale2=1.0;
                            let mut primary_dead=0.0;
                            let mut modifier_dead=0.0;

                            for j in (1 .. profile.len()).rev() {
                                let profile3 = profile.get(0..j).unwrap();
                                let mapping_scales=input_config.get_scales(profile3);

                                if let Some(x)=mapping_scales.get(&mapping).cloned() {
                                    scale2=x;
                                    break;
                                }
                            }

                            for j in (1 .. profile.len()).rev() {
                                let profile3 = profile.get(0..j).unwrap();
                                let mapping_deads=input_config.get_deads(profile3);

                                if let Some(x)=mapping_deads.get(&mapping).cloned() {
                                    primary_dead=x.0;
                                    modifier_dead=x.1;
                                    break;
                                }
                            }

                            bindings_out.insert(k, (scale*scale2,primary_dead,modifier_dead));
                        }
                    }
                }
            }
        }

        input_map.bindings_updated=true;
        // println!("input setup ok2");
        // println!("{:?}",input_map.owner_bindings);
    }
}