
#![allow(dead_code)]

// use bevy::asset::Handle;
use bevy::ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
// use super::assets::*;
use super::values::InputData;
use bevy_axis_input::{self as axis_input, Binding};

#[derive(Resource,)]
pub struct InputConfig<M:Send+Sync+Hash> {
    pub default_file : PathBuf,
    pub user_file : PathBuf,

    pub default_data:InputData<M>,
    pub user_data:InputData<M>,
    pub last_user_data:InputData<M>,

    // pub modified:bool,
    pub unapplied:bool,
    pub unsaved:bool,

    pub do_save : bool,
    pub do_apply : bool,

    // pub owner_profiles : HashMap<i32,HashSet<Vec<String>>>, //[owner]=profiles
}

impl<M:Send+Sync+Hash+Clone> Default for InputConfig<M> {
    fn default() -> Self {
        Self {
            default_file: Default::default(),
            user_file: Default::default(),

            default_data: Default::default(),
            user_data: Default::default(),

            last_user_data: Default::default(),
            // owner_profiles: Default::default(),

            // modified:false,
            unapplied:false,
            unsaved:false,

            do_save : false,
            do_apply : false,
        }
    }
}
/*
* don't bother with implementing funcs for repeats, excludes, deads, can just have the user set them from the files
*/

impl<M:Send+Sync+Hash+Eq+Clone> InputConfig<M> {
    //
    pub fn apply(&mut self) {
        if self.unapplied && !self.do_apply {
            // self.modified=false;
            self.unapplied=false;
            self.do_apply=true;
        }
    }

    pub fn save(&mut self) {
        if self.unsaved && !self.do_save {
            if self.unapplied {
                self.unapplied=false;
                self.do_apply=true;
            }

            self.unsaved=false;
            self.do_save=true;

            self.last_user_data=self.user_data.clone();
        }
    }

    pub fn restore(&mut self) {
        if self.unapplied || self.unsaved {
            if !self.unapplied {
                self.do_apply=true;
            }

            self.unapplied=false;
            self.unsaved=false;
            self.user_data=self.last_user_data.clone();
        }
    }

    pub fn is_applied(&self) -> bool {
        !self.unapplied
    }

    pub fn is_saved(&self) -> bool {
        !self.unsaved
    }

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
        let mut h=self.user_data.scales.get(&profile).cloned().unwrap_or_default();

        for (profile,&scale) in self.default_data.scales.get(&profile).map(|x|x.iter()).unwrap_or_default() {
            if !h.contains_key(profile) {
                h.insert(profile.clone(), scale);
            }
        }

        h
    }

    pub fn get_deads<P,S>(& self,profile : P) -> HashMap<M,(f32,f32)>
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();
        let mut h=self.user_data.deads.get(&profile).cloned().unwrap_or_default();

        for (profile,&dead) in self.default_data.deads.get(&profile).map(|x|x.iter()).unwrap_or_default() {
            if !h.contains_key(profile) {
                h.insert(profile.clone(), dead);
            }
        }

        h
    }

    pub fn get_bindings<P,S>(& self,profile : P) -> HashMap<M,Vec<(Vec<axis_input::Binding>,f32)>>
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile:Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();
        let mut h=self.user_data.bindings.get(&profile).cloned().unwrap_or_default();

        for (profile,binding_scales) in self.default_data.bindings.get(&profile).map(|x|x.iter()).unwrap_or_default() {
            if !h.contains_key(profile) {
                h.insert(profile.clone(), binding_scales.clone());
            }
        }

        h
    }

    pub fn get_owners_profiles(& self) -> HashMap<i32,HashSet<Vec<String>>> {
        let mut h=self.user_data.owners_profiles.clone();

        for (&profile,profiles) in self.default_data.owners_profiles.iter() {
            if !h.contains_key(&profile) {
                h.insert(profile, profiles.clone());
            }
        }

        h
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

    pub fn get_owner_profiles(self,owner:i32) -> HashSet<Vec<String>> {
        self.user_data.owners_profiles.get(&owner).or_else(||self.default_data.owners_profiles.get(&owner))
            .cloned().unwrap_or_default()
    }

    //
    pub fn owner_insert_profile<P,S>(&mut self,owner:i32,profile : P)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile: Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        if !self.user_data.owners_profiles.contains_key(&owner) {
            let x=self.default_data.owners_profiles.get(&owner).cloned().unwrap_or_default();
            self.user_data.owners_profiles.insert(owner,x);
        }

        self.user_data.owners_profiles.entry(owner).or_default().insert(profile);

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
    }

    pub fn owner_remove_profile<P,S>(&mut self,owner:i32,profile : P)
    where
        P:IntoIterator<Item = S>,
        S:AsRef<str>,
    {
        let profile: Vec<String>=profile.into_iter().map(|x|x.as_ref().to_string()).collect();

        let user_owner_profiles=self.user_data.owners_profiles.get_mut(&owner);

        if user_owner_profiles.as_ref().map(|x|x.contains(&profile)).unwrap_or_default() {
            user_owner_profiles.unwrap().remove(&profile);
        } else if user_owner_profiles.is_none() {
            if let Some(default_owner_profiles)=self.default_data.owners_profiles.get(&owner) {
                if default_owner_profiles.contains(&profile) {
                    let mut tmp_owner_profiles=default_owner_profiles.clone();
                    tmp_owner_profiles.remove(&profile);
                    self.user_data.owners_profiles.insert(owner, default_owner_profiles.clone());
                }
            }
        }

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
    }

    pub fn owner_clear_profiles(&mut self,owner:i32) {
        if let Some(user_owner_profiles)=self.user_data.owners_profiles.get_mut(&owner) {
            user_owner_profiles.clear();
        } else if self.default_data.owners_profiles.contains_key(&owner) {
            self.user_data.owners_profiles.insert(owner, Default::default());
        }

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
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

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
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

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
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

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
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

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
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

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
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

        if user_bindings.len() < binding_ind+1 {
            user_bindings.resize(binding_ind+1, Default::default());
        }

        user_bindings[binding_ind]=(bindings.into_iter().collect(),scale);

        //
        // self.modified=true;
        self.unapplied=true;
        self.unsaved=true;
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
