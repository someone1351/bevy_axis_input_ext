use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bevy_axis_input as axis_input;

#[derive(Debug, Clone)] //Default
pub struct InputData<M:Send+Sync+Hash> {
    pub includes:HashSet<axis_input::Binding>, //[binding]
    pub excludes:HashSet<axis_input::Binding>, //[binding]
    pub repeats:HashMap<M,(f32,f32)>, //[mapping]=(initial_delay,rate)
    pub bind_mode_deads:Option<(f32,f32)>,

    pub owners_profiles : HashMap<i32,HashSet<Vec<String>>>, //[owner]=profiles

    pub scales:HashMap<Vec<String>,HashMap<M,f32>>, //[profile][mapping]=scale
    pub deads:HashMap<Vec<String>,HashMap<M,(f32,f32)>>, //[profile][mapping]=(primary_dead,modifier_dead)
    pub bindings : HashMap<Vec<String>,HashMap<M,Vec<(Vec<axis_input::Binding>,f32)>>>, //[profile][mapping][mapping_ind][binding_ind]=(bindings,scale)

}

impl<M:Send+Sync+Hash> Default for InputData<M> {
    fn default() -> Self {
        Self {
            includes: Default::default(),
            excludes: Default::default(),
            repeats: Default::default(),
            bind_mode_deads:Default::default(),
            owners_profiles:Default::default(),
            scales: Default::default(),
            deads: Default::default(),
            bindings: Default::default(),
        }
    }
}