
#![allow(dead_code)]

// use bevy::asset::Handle;
use bevy::ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use std::str::FromStr;
// use super::assets::*;
use super::values::InputData;
use bevy_axis_input::{self as axis_input, Binding};

#[derive(Resource,)]
pub struct InputConfig<M:Send+Sync+Hash+PartialEq+Eq+FromStr> {
    pub default_file : PathBuf,
    pub user_file : PathBuf,
    // pub default_asset:Handle<InputAsset>,
    // pub user_asset:Handle<InputAsset>,
    // pub default_loaded:bool,
    // pub user_loaded:bool,
    pub default_data:InputData<M>,
    pub user_data:InputData<M>,
    // pub cur_data:InputData<M>,

    pub config_updated : bool,
    pub bindings_updated : bool,
    // pub user_data_addeds : HashMap<(M,Vec<axis_input::Binding>),(f32,f32,f32)>,
    // pub user_data_removeds : HashSet<(M,Vec<axis_input::Binding>),(f32,f32,f32)>,



    // pub excludes:HashSet<axis_input::Binding>, //[binding]
    // pub repeats:HashMap<M,f32>, //[mapping]=repeat
    // pub inverts:HashMap<Vec<String>,HashMap<M,bool>>, //[profile][mapping]=invert
    // pub scales:HashMap<Vec<String>,HashMap<M,f32>>, //[profile][mapping]=scale

    // //[profile][mapping][bind_ind]=(bindings,scale,primary_dead,modfier_dead)
    // pub bindings : HashMap<Vec<String>,HashMap<M,Vec<(Vec<axis_input::Binding>,f32,f32,f32)>>>,
    // pub owner_bindings : HashMap<i32,HashMap<(M,Vec<axis_input::Binding>),(f32,f32,f32)>>, //[owner][mapping,bindings]=(scale,primary_dead,modifier_dead)
    pub owner_profiles : HashMap<i32,HashSet<Vec<String>>>, //[owner]=profiles
}

impl<M:Send+Sync+Hash+PartialEq+Eq+FromStr+Clone> Default for InputConfig<M> {
    fn default() -> Self {
        Self {
            default_file: Default::default(),
            user_file: Default::default(),
            default_data: Default::default(),
            user_data: Default::default(),
            config_updated: false,
            bindings_updated: false,
            owner_profiles: Default::default(),
        }
    }
}
/*
* don't bother with implementing funcs for repeats, excludes, deads, can just have the user set them from the files
*/

impl<M:Send+Sync+Hash+PartialEq+Eq+FromStr+Clone> InputConfig<M> {

    //
    pub fn get_excludes(&self) -> HashSet<Binding> {
        if !self.user_data.excludes.is_empty() {
            self.user_data.excludes.clone()
        } else {
            self.default_data.excludes.clone()
        }
    }
    pub fn get_includes(&self) -> HashSet<Binding> {
        if !self.user_data.includes.is_empty() {
            self.user_data.includes.clone()
        } else {
            self.default_data.includes.clone()
        }
    }
    pub fn get_mapping_repeats(&self) -> HashMap<M, (f32, f32)> {
        let mut h= self.user_data.repeats.clone();

        for (m,&r) in self.default_data.repeats.iter() {
            if !h.contains_key(m) {
                h.insert(m.clone(), r);
            }
        }

        h
    }

    pub fn get_bind_mode_deads(&self) -> Option<(f32, f32)> {
        self.user_data.bind_mode_deads.or_else(||self.default_data.bind_mode_deads)
    }

    //
    pub fn get_scales<P,S>(&self,profile : P) -> HashMap<M,f32>
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();
        self.user_data.scales.get(&profile).or_else(||self.default_data.scales.get(&profile)).cloned().unwrap_or_default()

    }

    //
    pub fn get_deads<P,S>(& self,profile : P) -> HashMap<M,(f32,f32)>
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();
        self.user_data.deads.get(&profile).or_else(||self.default_data.deads.get(&profile)).cloned().unwrap_or_default()
    }

    pub fn get_bindings<P,S>(& self,profile : P) -> HashMap<M,Vec<(Vec<axis_input::Binding>,f32)>>
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();
        self.user_data.bindings.get(&profile).or_else(||self.default_data.bindings.get(&profile)).cloned().unwrap_or_default()
    }

    //
    pub fn owner_insert_profile<P,S>(&mut self,owner:i32,profile : P)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile: Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();
        self.owner_profiles.entry(owner).or_default().insert(profile);

        // self.config_updated=true;
        self.bindings_updated=true;
    }

    pub fn owner_remove_profile<P,S>(&mut self,owner:i32,profile : P)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {

        if let Some(profiles)=self.owner_profiles.get_mut(&owner) {
            let profile: Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();
            profiles.remove(&profile);

            if profiles.is_empty() {
                self.owner_profiles.remove(&owner).unwrap();
            }
        }

        // self.config_updated=true;
        self.bindings_updated=true;
    }

    pub fn owner_clear_profiles(&mut self,owner:i32) {
        self.owner_profiles.remove(&owner);

        // self.config_updated=true;
        self.bindings_updated=true;
    }

    //
    pub fn new_scales_profile<P,F,S,T>(&mut self,profile:P, from_profile : Option<F>, )
    where
        P:IntoIterator<Item = S>,
        F:IntoIterator<Item = T>,
        S:AsRef<str>,
        T:AsRef<str>,
    {
        let from_profile:Option<Vec<String>>=from_profile.map(|y|y.into_iter().map(|x|x.as_ref().to_string()).collect());
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        let new_scales=if let Some(from_profile)=from_profile {
            if let Some(x)=self.user_data.scales.get(&from_profile) {
                Some(x)
            } else if let Some(x)=self.default_data.scales.get(&from_profile) {
                Some(x)
            } else {
                None
            }
        } else {
            None
        }.cloned().unwrap_or_default();

        self.user_data.scales.insert(profile,new_scales);
        self.config_updated=true;
    }

    pub fn new_deads_profile<P,F,S,T>(&mut self,profile:P, from_profile : Option<F>, )
    where
        P:IntoIterator<Item = S>,
        F:IntoIterator<Item = T>,
        S:AsRef<str>,
        T:AsRef<str>,
    {
        let from_profile:Option<Vec<String>>=from_profile.map(|y|y.into_iter().map(|x|x.as_ref().to_string()).collect());
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        let new_deads=if let Some(from_profile)=from_profile {
            if let Some(x)=self.user_data.deads.get(&from_profile) {
                Some(x)
            } else if let Some(x)=self.default_data.deads.get(&from_profile) {
                Some(x)
            } else {
                None
            }
        } else {
            None
        }.cloned().unwrap_or_default();

        self.user_data.deads.insert(profile,new_deads);
        self.config_updated=true;
    }

    pub fn new_bindings_profile<P,F,S,T>(&mut self,profile:P, from_profile : Option<F>, )
    where
        P:IntoIterator<Item = S>,
        F:IntoIterator<Item = T>,
        S:AsRef<str>,
        T:AsRef<str>,
    {
        let from_profile:Option<Vec<String>>=from_profile.map(|y|y.into_iter().map(|x|x.as_ref().to_string()).collect());
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        let new_bindings=if let Some(from_profile)=from_profile {
            if let Some(x)=self.user_data.bindings.get(&from_profile) {
                Some(x)
            } else if let Some(x)=self.default_data.bindings.get(&from_profile) {
                Some(x)
            } else {
                None
            }
        } else {
            None
        }.cloned().unwrap_or_default();

        self.user_data.bindings.insert(profile,new_bindings);
        self.config_updated=true;
    }

    //
    pub fn set_scale<P,S>(&mut self,profile : P,mapping:M,scale:f32)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        if !self.user_data.scales.contains_key(&profile) {
            if let Some(default_scales)=self.default_data.scales.get(&profile).cloned() {
                self.user_data.scales.insert(profile.clone(), default_scales);
            }
        }

        let scales=self.user_data.scales.entry(profile).or_default();
        scales.insert(mapping,scale);

        self.config_updated=true;
    }

    pub fn set_dead<P,S>(&mut self,profile : P,mapping:M,primary_dead:f32,modifier_dead:f32)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        if !self.user_data.deads.contains_key(&profile) {
            if let Some(default_deads)=self.default_data.deads.get(&profile).cloned() {
                self.user_data.deads.insert(profile.clone(), default_deads);
            }
        }

        let deads=self.user_data.deads.entry(profile).or_default();
        deads.insert(mapping,(primary_dead,modifier_dead));

        self.config_updated=true;
    }

    pub fn set_binding<P,S,B>(&mut self,profile : P,mapping:M,binding_ind:usize,bindings:B,scale:f32)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
        B:IntoIterator<Item = axis_input::Binding>,
    {
        let profile: Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        if !self.user_data.bindings.contains_key(&profile) {
            if let Some(default_bindings)=self.default_data.bindings.get(&profile).cloned() {
                self.user_data.bindings.insert(profile.clone(), default_bindings);
            }
        }

        let user_bindings=self.user_data.bindings
            .entry(profile).or_default()
            .entry(mapping).or_default();

        user_bindings.resize(binding_ind+1, Default::default());
        user_bindings[binding_ind]=(bindings.into_iter().collect(),scale);

        self.config_updated=true;
    }

    //
    pub fn get_scale<P,S>(& self,profile : P,mapping:M) -> Option<f32>
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        self.user_data.scales.get(&profile).or_else(||self.default_data.scales.get(&profile))
            .and_then(|profile_scales|profile_scales.get(&mapping)).cloned()

    }

    pub fn get_dead<P,S>(& self,profile : P,mapping:M) -> Option<(f32,f32)>
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        self.user_data.deads.get(&profile).or_else(||self.default_data.deads.get(&profile))
            .and_then(|profile_deads|profile_deads.get(&mapping)).cloned()

    }

    pub fn get_binding<P,S>(&self,profile : P,mapping:M,binding_ind:usize) -> (Vec<axis_input::Binding>,f32)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile=profile.into_iter().map(|x|x.as_ref().to_string()).collect::<Vec<_>>();

        self.user_data.bindings.get(&profile).or_else(||self.default_data.bindings.get(&profile))
            .and_then(|x|x.get(&mapping))
            .and_then(|x|x.get(binding_ind))
            .cloned()
            .unwrap_or_default()
    }





}

   // pub fn clear_bindings<P,S,B>(&mut self,profile : P,mapping:M)
    // where
    //     P:IntoIterator<Item = S>,
    //     S:AsRef<str>,
    //     B:IntoIterator<Item = axis_input::Binding>,
    // {
    //     let profile=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

    //     let user_bindings=self.user_data.bindings
    //         .entry(profile).or_default()
    //         .entry(mapping).or_default();

    //     user_bindings.clear();

    //     self.config_updated=true;
    // }
