
// use super::assets::*;
use super::resources::*;
use super::values::*;

use std::str::FromStr;
use std::{cmp::Ordering, collections::HashMap, hash::Hash, path::Path};
use bevy_axis_input as axis_input;
// use std::fmt::Debug;

pub fn save_input_data<M:Send+Sync+Hash+Eq+std::any::Any+Clone+FromStr+ToString>(data:&InputData<M>,path:&Path) -> bool {
    let mut writer = conf_lang::Writer::new();

    //excludes
    if !data.excludes.is_empty() {
        writer.record(0).param("exclude");

        for binding in data.excludes.iter() {
            writer.record(1).param(binding.clone());
        }

        writer.newline(0);
    }

    //repeats
    if !data.repeats.is_empty() {
        writer.record(0).param("repeat");

        for (mapping,&(init,delay)) in data.repeats.iter() {
            writer.record(1).param(mapping.clone()).param(init).param(delay);
        }

        writer.newline(0);
    }

    //deads
    if !data.deads.is_empty() {
        for (profile,mapping_deads) in data.deads.iter() {
            writer.record(0).param("dead").param_dquotes(false,profile);

            for (mapping, &(primary_dead,modifier_dead)) in mapping_deads.iter() {
                writer.record(1).param(mapping.clone()).param(primary_dead).param(modifier_dead);
            }
        }

        writer.newline(0);
    }

    //scales
    if !data.scales.is_empty() {
        for (profile,mapping_scales) in data.scales.iter() {
            writer.record(0).param("scale").param_dquotes(false,profile);

            for (mapping, &scale) in mapping_scales.iter() {
                writer.record(1).param(mapping.clone()).param(scale);
            }
        }

        writer.newline(0);
    }

    //bindings
    if !data.bindings.is_empty() {
        for (profile,mapping_bindings) in data.bindings.iter() {
            writer.record(0).param("binding").param_dquotes(false,profile);

            for (mapping, binding_group_scales) in mapping_bindings.iter() {
                writer.record(1).param(mapping.clone());

                for (binding_group,scale) in binding_group_scales {
                    writer.record(2).params(binding_group).param(scale);
                }
            }
        }

        writer.newline(0);
    }

    //owner_profiles
    if !data.owners_profiles.is_empty() {
        for (&owner,profiles) in data.owners_profiles.iter() {
            writer.record(0).param("owner").param(owner);

            for profile in profiles {
                writer.record(1).param_dquotes(false, profile);
            }
        }

        writer.newline(0);
    }

    //
    let mut dir=path.to_path_buf();
    let mut dir_ok=true;

    if dir.pop() {
        dir_ok=dir.try_exists().unwrap_or(false);
    }

    if !dir_ok {
        dir_ok=std::fs::create_dir_all(&dir).is_ok();
    }

    if !dir_ok {
        eprintln!("Cannot save to path: {path:?}");
        return false;
    }

    //
    let Ok(mut output) = std::fs::File::create(path) else {
        eprintln!("Cannot create input_conf file: {path:?}");
        return false;
    };

    use std::io::Write;

    if write!(output, "{}",writer).is_err() {
        eprintln!("Cannot write input_conf file: {path:?}");
        return false;
    }

    //
    true
}

pub fn get_input_data<M:Send+Sync+Hash+PartialEq+Eq+std::str::FromStr+std::any::Any+Clone>(conf:&conf_lang::Conf,data:&mut InputData<M>) { //+Debug
    conf.root().walk(|walk|{
        // println!("hmm {:?}",walk.record().node_label());
        match walk.record().node_label().unwrap_or("") {

            "exclude" => {
                let binding= walk.record().value(0).get_parsed::<axis_input::Binding>().unwrap();
                data.excludes.insert(binding);
            }
            "repeat" => {
                let mapping= walk.record().value(0).get_parsed::<M>().unwrap();
                let initial_delay= walk.record().value(1).get_parsed::<f32>().unwrap();
                let rate= walk.record().value(2).get_parsed::<f32>().unwrap();
                data.repeats.insert(mapping,(initial_delay,rate));
            }
            // "invert" => {
            //     let profile=walk.parent().record().values().map(|x|x.to_string()).collect();
            //     let mapping=walk.record().value(0).get_parsed::<M>().unwrap();
            //     let invert= walk.record().value(1).get_parsed::<bool>().unwrap();
            //     data.inverts.entry(profile).or_default().insert(mapping, invert);
            // }
            "dead" => {
                let profile: Vec<String>=walk.parent().record().values().map(|x|x.to_string()).collect();
                let mapping=walk.record().value(0).get_parsed::<M>().unwrap();
                let primary_dead= walk.record().value(1).get_parsed::<f32>().unwrap();
                let modifier_dead= walk.record().value(2).get_parsed::<f32>().unwrap_or_default();
                data.deads.entry(profile).or_default().insert(mapping, (primary_dead,modifier_dead));
            }

            "scale" => {
                let profile: Vec<String>=walk.parent().record().values().map(|x|x.to_string()).collect();
                let mapping=walk.record().value(0).get_parsed::<M>().unwrap();
                let scale= walk.record().value(1).get_parsed::<f32>().unwrap();
                data.scales.entry(profile).or_default().insert(mapping, scale);
            }

            "binding" => {
                let profile: Vec<String>=walk.ancestor(1).record().values().map(|x|x.to_string()).collect();
                let mapping=walk.parent().record().first().get_parsed::<M>().unwrap();
                let bindings: Vec<bevy_axis_input::Binding>=walk.record().param_group(0).values().map(|x|x.get_parsed().unwrap()).collect();
                let scale=walk.record().param_group(1).first().get_parsed::<f32>().unwrap_or(1.0);

                // println!("b {mapping:?} {bindings:?}");

                data.bindings
                    .entry(profile).or_default()
                    .entry(mapping).or_default()
                    .push((bindings,scale,));

            }
            "bind_mode_dead" => {
                let a=walk.record().value(0).get_parsed::<f32>().unwrap();
                let b=walk.record().value(1).get_parsed::<f32>().unwrap();
                data.bind_mode_deads = Some((a,b));
            }
            "owner" => {
                let owner=walk.record().first().get_parsed::<i32>().unwrap();
                data.owners_profiles.entry(owner).or_default().clear();
                // println!("owner {owner:?}");
            }
            "owner_profile" => {
                let owner=walk.parent().record().first().get_parsed::<i32>().unwrap();
                let profile:Vec<String> = walk.record().values().map(|x|x.str().to_string()).collect();
                // println!("owner {owner:?} profile {profile:?}");
                data.owners_profiles.get_mut(&owner).unwrap().insert(profile);
            }
            _ =>{}
        }
    }).unwrap();

    // println!("load {:?}",data.bindings);
}


pub fn create_def<M:std::str::FromStr+Sync+Send+std::any::Any>() -> conf_lang::Def {
    conf_lang::Def::new()
        .branch("root_branch")
            .tags(["exclude"]).entry_children("branch_exclude")
            .tags(["include"]).entry_children("branch_include")
            .tags(["repeat"]).entry_children("branch_repeat")
            .tags(["scale"]).entry_children("branch_scale")
                .group().param_any()
                .group().grepeat().goptional().param_any()
            // .tags(["invert"]).entry_children("branch_invert")
            //     .group().param_any()
            //     .group().grepeat().goptional().param_any()
            .tags(["dead"]).entry_children("branch_dead")
                .group().param_any()
                .group().grepeat().goptional().param_any()
            .tags(["binding"]).entry_children("branch_mapping")
                .group().param_any()
                .group().grepeat().goptional().param_any()
            // .tags(["attrib"]).entry_children("branch_attrib")
            //     .group().param_any()
            //     .group().grepeat().goptional().param_any()
            .tags(["bind_mode_dead"]).entry().elabel("bind_mode_dead").param_parse::<f32>().param_parse::<f32>()
            .tags(["owner"]).entry_children("branch_owner").elabel("owner")
                .param_parse::<i32>()

        .branch("branch_exclude")
            .tagless().entry().elabel("exclude")
                .param_parse::<axis_input::Binding>()

        .branch("branch_include")
            .tagless().entry().elabel("include")
                .param_parse::<axis_input::Binding>()

        .branch("branch_repeat")
            .tagless().entry().elabel("repeat")
                .param_parse::<M>()
                .param_parse::<f32>()
                .param_parse::<f32>()

        .branch("branch_scale")
            .tagless().entry().elabel("scale")
                .param_parse::<M>()
                .param_parse::<f32>()

        .branch("branch_owner")
            .tagless().entry().elabel("owner_profile").grepeat()
                .param_any()

        // .branch("branch_invert")
        //     .tagless()
        //         .entry().elabel("invert")
        //             .param_parse::<M>()
        //             .param_parse::<bool>()

        .branch("branch_dead")
            .tagless().entry().elabel("dead")
                .param_parse::<M>()
                .param_parse::<f32>()
                .param_parse::<f32>()
        // .branch("branch_attrib")
        //     .tagless()
        //         .entry().elabel("attrib")
        //             .param_parse::<M>()
        //             .param_optional()
        //             .param_parse::<f32>()
        //             // .param_parse::<f32>()
        //             // .param_parse::<f32>()
        .branch("branch_mapping")
            .tagless()
                .entry_children("branch_binding").elabel("mapping")
                    // .group()
                    .param_parse::<M>()
                    // // // .param_optional()
                    // // // .param_func(|x|{match x {"+"=>Some('+'),"-"=>Some('-'),_=>None}})
                    // // .group().grepeat().goptional().param_any()
                    // .param_optional()
                    // .param_parse::<f32>()
                    // .param_parse::<f32>()
                    // .param_parse::<f32>()


        .branch("branch_binding")
            .tagless()
                .entry().elabel("binding")
                    .group().grepeat()
                        .param_parse::<axis_input::Binding>()
                    .group() //.goptional()
                        .param_optional()
                        .param_parse::<f32>()
                        // .param_parse::<f32>()
                        // .param_parse::<f32>()

}

pub fn load_input_conf<M:std::str::FromStr+Sync+Send+std::any::Any>(path:&Path) -> Option<conf_lang::Conf> {
    let src = std::fs::read_to_string(path);
    let def=create_def::<M>();

    if let Ok(src)=src {

        match def.get_root_branch().parse(&src,true,Some(path)) {
            Ok(conf) => {
                Some(conf)
            },
            Err(e) => {
                eprintln!("{}",e.msg(Some(&src)));
                None
            }
        }
    } else {
        eprintln!("Failed to read input config file: {path:?}",);
        None
    }
}

pub fn apply_input_map<M:Send+Sync+Hash+PartialEq+Eq+Clone>(
    input_config:&mut InputConfig<M>,
    owner_bindings: &mut HashMap<i32, HashMap<(M, Vec<axis_input::Binding>), (f32, f32, f32)>>,
) {
    owner_bindings.clear();

    for (owner,profiles) in input_config
        // .owner_profiles.iter()
        .get_owners_profiles()

    {
        let bindings_out=owner_bindings.entry(owner).or_default();

        //
        let mut profiles = profiles.clone();

        for profile in profiles.clone() {
            for i in (1 .. profile.len()).rev() {
                profiles.insert(profile.get(0..i).unwrap().to_vec());
            }
        }

        //
        let mut profiles: Vec<Vec<String>> = profiles.iter().map(|x|x.clone()).collect::<Vec<_>>();

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
                let mut binding_group_scales2: HashMap<Vec<bevy_axis_input::Binding>, f32> = HashMap::new();

                for (binding_group,scale) in binding_group_scales {
                    *binding_group_scales2.entry(binding_group).or_default()+=scale;
                }

                for (binding_group,scale) in binding_group_scales2 {
                    let k: (M, Vec<bevy_axis_input::Binding>) =(mapping.clone(),binding_group.iter().map(|x|x.clone()).collect::<Vec<_>>());
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
}