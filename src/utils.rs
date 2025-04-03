
// use super::assets::*;
// use super::resources::*;
use super::values::*;

use std::{hash::Hash, path::Path};
use bevy_axis_input as axis_input;
// use std::fmt::Debug;

pub fn save_input_data<M:Send+Sync+Hash+PartialEq+Eq+std::str::FromStr+std::any::Any+Clone+ToString>(data:&InputData<M>,path:&Path) -> bool {
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
                let profile=walk.parent().record().values().map(|x|x.to_string()).collect();
                let mapping=walk.record().value(0).get_parsed::<M>().unwrap();
                let primary_dead= walk.record().value(1).get_parsed::<f32>().unwrap();
                let modifier_dead= walk.record().value(2).get_parsed::<f32>().unwrap_or_default();
                data.deads.entry(profile).or_default().insert(mapping, (primary_dead,modifier_dead));
            }

            "scale" => {
                let profile=walk.parent().record().values().map(|x|x.to_string()).collect();
                let mapping=walk.record().value(0).get_parsed::<M>().unwrap();
                let scale= walk.record().value(1).get_parsed::<f32>().unwrap();
                data.scales.entry(profile).or_default().insert(mapping, scale);
            }

            "binding" => {
                let profile=walk.ancestor(1).record().values().map(|x|x.to_string()).collect();
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
            .tags(["scale"]).entry_children("branch_scale").elabel("profile")
                .group().param_any()
                .group().grepeat().goptional().param_any()
            // .tags(["invert"]).entry_children("branch_invert").elabel("profile")
            //     .group().param_any()
            //     .group().grepeat().goptional().param_any()
            .tags(["dead"]).entry_children("branch_dead").elabel("profile")
                .group().param_any()
                .group().grepeat().goptional().param_any()
            .tags(["binding"]).entry_children("branch_mapping").elabel("profile")
                .group().param_any()
                .group().grepeat().goptional().param_any()
            // .tags(["attrib"]).entry_children("branch_attrib").elabel("profile")
            //     .group().param_any()
            //     .group().grepeat().goptional().param_any()
            .tags(["bind_mode_dead"]).entry().elabel("bind_mode_dead").param_parse::<f32>().param_parse::<f32>()

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