

use bevy::prelude::*;
use bevy_axis_input::{self as axis_input, Binding,  };
use bevy_axis_input_ext as axis_input_ext;

use serde::Deserialize;


#[derive(Clone,Debug,Deserialize,Hash,PartialEq,Eq,Ord,PartialOrd)]
pub enum Mapping {
    X,Y,
    Quit,
    MenuSelect,
    MenuCancel,
    MenuUp,
}

impl std::str::FromStr for Mapping {
    type Err = ron::de::SpannedError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { ron::de::from_str::<Self>(s) }
}

impl ToString for Mapping {
    fn to_string(&self) -> String {
        format!("{:?}",self)
    }
}

#[derive(Resource,Default)]
struct Menu {
    cur_index : i32,
    pressed : Option<i32>,
    x_val : f32,
    y_val : f32,
    in_bind_mode:bool,
}
fn main() {
    let mut app = App::new();

    app
        .add_plugins((
            DefaultPlugins
            //     .set(AssetPlugin {watch_for_changes_override:Some(true), ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "some input map".into(),
                        resolution: (800, 600).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
                axis_input::InputMapPlugin::<Mapping>::default(),

                axis_input_ext::InputConfigPlugin::<Mapping> {
                    default_file_path: "config".to_string(),
                    user_file_path: "config".to_string(),
                    ..Default::default()
                },
        ))

        .init_resource::<Menu>()

        .add_systems(Startup, (
            setup_input,
            setup_camera, setup_menu,
        ))
        .add_systems(PreUpdate, ( update_input, ).after(axis_input::InputMapSystem))
        .add_systems(Update, ( show_menu, ))
        ;

    app.run();
}

fn setup_input(
    mut input_config: ResMut<axis_input_ext::InputConfig<Mapping>>,
) {
    input_config.owner_insert_profile(0, ["ui"]);
    input_config.owner_insert_profile(0, ["game"]);
}

// #[derive(Resource)]
// struct CurBindModeBinds(Vec<Binding>);

fn update_input(
    mut input_map_event: MessageReader<axis_input::InputMapMessage<Mapping>>,
    mut exit: MessageWriter<AppExit>,
    mut menu : ResMut<Menu>,
    mut input_map: ResMut<axis_input::InputMap<Mapping>>,
    mut input_config: ResMut<axis_input_ext::InputConfig<Mapping>>,
    mut commands: Commands,
    gamepad_owner_query: Query<(Entity,&axis_input::GamepadOwner,)>,
    gamepad_ownerless_query:Query<Entity,(With<Gamepad>,Without<axis_input::GamepadOwner>)>,
) {

    for entity in gamepad_ownerless_query.iter() {
        commands.entity(entity).insert(axis_input::GamepadOwner(0));
    }


    for ev in input_map_event.read() {
        match ev.clone() {

            axis_input::InputMapMessage::ValueChanged { mapping:Mapping::X, val, .. } => {
                menu.x_val=val;
            }
            axis_input::InputMapMessage::ValueChanged { mapping:Mapping::Y, val, .. } => {
                menu.y_val=val;
            }
            axis_input::InputMapMessage::JustPressed{mapping:Mapping::Quit, ..} => {
                exit.write(AppExit::Success);
            }
            axis_input::InputMapMessage::JustPressed{mapping:Mapping::MenuUp, dir, ..}
                |axis_input::InputMapMessage::Repeat { mapping:Mapping::MenuUp, dir, .. }
                if !menu.in_bind_mode
            => {
                menu.cur_index-=dir;
                let n= 4;
                if menu.cur_index<0 {menu.cur_index=n-1;}
                if menu.cur_index==n {menu.cur_index=0;}
                menu.pressed=None;
            }
            axis_input::InputMapMessage::JustPressed{mapping:Mapping::MenuSelect, ..} => {
                menu.pressed=Some(menu.cur_index);
            }
            axis_input::InputMapMessage::JustReleased{mapping:Mapping::MenuSelect, ..} => {
                if let Some(pressed)=menu.pressed {
                    match pressed {
                        0..=2 => { //X+ X- Y
                            if let Ok((entity,_owner)) = gamepad_owner_query.single() {
                                commands.entity(entity).entry().or_insert(axis_input::GamepadBindMode(true));
                            }

                            input_map.kbm_bind_mode=true;
                            menu.in_bind_mode=true;
                            println!("bind mode start");
                        }
                        3 => { //Exit
                            exit.write(AppExit::Success);
                        }
                        _ =>{}
                    }
                }
                menu.pressed=None;
            }

            axis_input::InputMapMessage::BindPressed { .. } => {
            }
            axis_input::InputMapMessage::BindReleased {  bindings, .. } => {
                // input_map.set_bind_mode_devices([]);
                // input_map.bind_mode_devices.clear(); //todo!

                if let Ok((entity,_owner)) = gamepad_owner_query.single()

                {
                    commands.entity(entity).entry::<axis_input::GamepadBindMode>().and_modify(|mut c|{c.0=false;});
                }
                input_map.kbm_bind_mode=false;

                menu.in_bind_mode=false;

                //
                match menu.cur_index {
                    0 => {//x+
                        input_config.set_binding(["game"], Mapping::X, 0, bindings.clone(), 1.0);
                    },
                    1 => {//x-
                        input_config.set_binding(["game"], Mapping::X, 1, bindings.clone(), -1.0);
                    },
                    2 => { //y
                        input_config.set_binding(["game"], Mapping::Y, 0, bindings.clone(), 1.0);
                    },
                    _ =>{
                        continue;
                    }
                }

                input_config.save();

            }
            axis_input::InputMapMessage::JustPressed{mapping:Mapping::MenuCancel, ..} => {
                if menu.in_bind_mode {
                    if let Ok((entity,_owner)) = gamepad_owner_query.single() {
                        commands.entity(entity).entry::<axis_input::GamepadBindMode>().and_modify(|mut c|{c.0=false;});
                    }

                    input_map.kbm_bind_mode=false;
                    menu.in_bind_mode=false;
                } else {
                    //
                    match menu.cur_index {
                        0 => {//x+
                            input_config.set_binding(["game"], Mapping::X, 0, vec![], 1.0);
                        },
                        1 => {//x-
                            input_config.set_binding(["game"], Mapping::X, 1, vec![], -1.0);
                        },
                        2 => { //y
                            input_config.set_binding(["game"], Mapping::Y, 0, vec![], 1.0);
                        },
                        _ =>{
                            continue;
                        }
                    }
                    input_config.save();
                }
            }

            _=>{}
        }
    }
}

fn setup_camera(mut commands: Commands) {
    // commands.spawn((Camera2d,));
    commands.spawn(Camera3d::default());
}

#[derive(Component)]
struct MenuItem(i32);

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("FiraMono-Medium.ttf");

    commands.spawn((
        Text::default(),
        TextLayout::new_with_justify(Justify::Center),
        Node {align_self:AlignSelf::Center,justify_self:JustifySelf::Center,..Default::default()},
    )).with_child((
        TextSpan::new("\"Press Up/Down to navigate, Enter to select, Escape to cancel/clear binding.\""),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 15.0,..default()},
    )).with_child((
        TextSpan::new("\n\n"),
        TextFont {font:font.clone(),font_size: 20.0,..default()},
    )).with_child((
    )).with_child((
        MenuItem(-1),
        TextSpan::new("values"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 20.0,..default()},
    )).with_child((
        TextSpan::new("\n"),
        TextFont {font:font.clone(),font_size: 20.0,..default()},
    )).with_child((
        MenuItem(0),
        TextSpan::new("b\n"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 20.0,..default()},
    )).with_child((
        MenuItem(1),
        TextSpan::new("b\n"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 20.0,..default()},
    )).with_child((
        MenuItem(2),
        TextSpan::new("b\n"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 20.0,..default()},
    )).with_child((
        MenuItem(3),
        TextSpan::new("Exit"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 20.0,..default()},
    ));
}

fn show_menu(
    mut marker_query: Query<(&MenuItem, &mut TextSpan, &mut TextColor)>,
    menu : Res<Menu>,
    input_config: Res<axis_input_ext::InputConfig<Mapping>>,

    mut bind_mode_chain : Local<Vec<Binding>>,

    mut input_map_event: MessageReader<axis_input::InputMapMessage<Mapping>>,
) {
    // let mut bind_mode_chain = Vec::new();
    for ev in input_map_event.read() {
        match ev.clone() {
            axis_input::InputMapMessage::BindPressed {  bindings, .. } => {
                *bind_mode_chain=bindings;
            }
            axis_input::InputMapMessage::BindReleased {   .. } => {
                bind_mode_chain.clear();
            }
            _=>{}
        }
	}

    for (item,mut text,mut col) in marker_query.iter_mut() {

        if item.0==menu.cur_index {
            col.0=Color::linear_rgb(1.0, 0.0, 0.0);
        } else {
            col.0=Color::linear_rgb(1.0,1.0,1.0);
        }

        if let Some(i)=menu.pressed {
            if item.0==i {
                col.0=Color::linear_rgb(0.8, 0.8, 0.0);
            } else {
                col.0=Color::linear_rgb(1.0,1.0,1.0);
            }
        }

        match item.0 {
            -1 => {
                text.0=format!("\"X={:.3}, Y={:.3}\"\n",menu.x_val,menu.y_val);
            }
            0 => {
                text.0=format!("Rebind X+ : {:?}\n",
                    if menu.in_bind_mode&&menu.cur_index==0 {
                        if bind_mode_chain.is_empty() {
                            "...".to_string()
                        } else {
                            format!("{:?}",bind_mode_chain.clone())
                        }
                    }else{
                        // format!("{:?}",cur_binds.x_pos)
                        format!("{:?}",input_config.get_binding(["game"], Mapping::X, 0).0)
                    }
                );
            }
            1 => {
                text.0=format!("Rebind X- : {:?}\n",
                    if menu.in_bind_mode&&menu.cur_index==1 {
                        if bind_mode_chain.is_empty() {
                            "...".to_string()
                        } else {
                            format!("{:?}",bind_mode_chain.clone())
                        }
                    }else{
                        // format!("{:?}",cur_binds.x_neg)
                        format!("{:?}",input_config.get_binding(["game"], Mapping::X, 1).0)
                    }
                );
            }
            2 => {
                text.0=format!("Rebind Y : {:?}\n",
                    if menu.in_bind_mode&&menu.cur_index==2 {
                        if bind_mode_chain.is_empty() {
                            "...".to_string()
                        } else {
                            format!("{:?}",bind_mode_chain.clone())
                        }
                    }else{
                        // format!("{:?}",cur_binds.y)
                        format!("{:?}",input_config.get_binding(["game"], Mapping::Y, 0).0)
                    }
                );
            }
            _ => {}
        }
    }

}
