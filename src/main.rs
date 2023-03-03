// Made for REL323
// Spring 2023

use bevy::prelude::*;
use bevy::text::Text2dBounds;
use bevy::render::camera::RenderTarget;
use std::collections::HashMap;
use rstar::{RTreeObject, PointDistance, AABB, RTree};
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::prelude::*;

#[derive(Resource)]
struct WorldState {
    cursor_pos: Vec2,
    transforming: Option<(Entity, Transform)>,
}

impl FromWorld for WorldState {
    fn from_world(_world: &mut World) -> Self {
        WorldState {
            cursor_pos: Vec2::default(),
            transforming: None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Action {
    Create(&'static str),
    Delete(&'static str),
}

const STARTING_KAMI: [&str;5]  = ["he_who", "she_who", "land", "ocean", "heaven"];

#[derive(Resource)]
struct KamiLookups {
    asset_name: HashMap<&'static str, &'static str>,
    full_name: HashMap<&'static str, &'static str>,
    combos: HashMap<(&'static str, &'static str), Vec<Action>>,
    //done: HashMap<&'static str, bool>,
}

impl FromWorld for KamiLookups {
    fn from_world(_world: &mut World) -> Self {
        //TODO just the basic kami right now, can be expanded
        let mut asset_name = HashMap::new();
        let mut full_name = HashMap::new();

        let to_insert = [
            ("ocean", "Ocean.png", "The Ocean"),
            ("land", "Land.png", "The Land"),
            ("heaven", "Heaven.png", "The Heavens"),
            ("he_who", "HeWho.png", "He Who Beckoned"),
            ("she_who", "SheWho.png", "She Who Beckoned"),
            ("leech", "Leech.png", "Leech Child"),
            ("bad_flame", "BadFlame.png", "Swift Burning Flame Man"),
            ("she_who_dead", "SheWhoDead.png", "She Who Beckoned (dead)"),
            ("he_who_dirty", "HeWhoDirty.png", "He Who Beckoned (dirty)"),
            ("rrm", "RRM.png", "Rushing Raging Man"),
            ("heaven_shining", "HeavenShining.png", "Heaven Shining"),
            ("moon_counting", "MoonCounting.png", "Moon Counting"),
        ];

        for (internal_name, a_name, f_name) in to_insert {
            asset_name.insert(internal_name, a_name);
            full_name.insert(internal_name, f_name);
        };

        let mut combos = HashMap::new();
        combos.insert(("he_who", "she_who"), vec![Action::Create("she_who_dead"), Action::Create("bad_flame"), Action::Create("leech"), Action::Delete("she_who")]);
        combos.insert(("he_who", "bad_flame"), vec![Action::Delete("bad_flame")]);
        combos.insert(("he_who_dirty", "bad_flame"), vec![Action::Delete("bad_flame")]);
        combos.insert(("he_who", "she_who_dead"), vec![Action::Create("he_who_dirty"), Action::Delete("he_who"), Action::Delete("she_who_dead")]);
        combos.insert(("he_who_dirty", "ocean"), vec![Action::Create("rrm"), Action::Create("moon_counting"), Action::Create("heaven_shining"), Action::Create("he_who"), Action::Delete("he_who_dirty")]);

        KamiLookups {
            asset_name,
            full_name,
            combos,
            //done: HashMap::default(),
        }
    }
}

#[derive(Resource)]
struct WidgetQueues {
    spawn_queue: Vec<(KamiWidget, Vec3)>,
    delete_queue: Vec<Entity>,
}

impl FromWorld for WidgetQueues {
    fn from_world(_world: &mut World) -> Self {
        WidgetQueues {
            spawn_queue: Vec::default(),
            delete_queue: Vec::default(),
        }
    }
}

#[derive(Component)]
struct KamiWidget {
    internal_name: &'static str
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LookupWidget {
    bounds: Rect,
    priority: f32,
    e: Entity,
}

impl PointDistance for LookupWidget {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        if self.contains_point(point) {
            return 0.0 
        } 
        let rect = self.bounds;
        let x_diff = (rect.min.x - point[0]).abs().min((point[0] - rect.max.x).abs());
        let x_diff = x_diff.min(0.0);
        let y_diff = (rect.min.y - point[1]).abs().min((point[1] - rect.max.y).abs());
        let y_diff = y_diff.min(0.0);
        return (x_diff * x_diff) + (y_diff * y_diff)
    }

    fn contains_point(&self, point: &[f32; 2]) -> bool {
        self.bounds.contains(Vec2::from_array(point.clone()))
    }
}

impl RTreeObject for LookupWidget {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope
    {
        AABB::from_corners(self.bounds.min.into(), self.bounds.max.into())
    }
}

#[derive(Resource)]
struct SpatialLookup {
    rtree: RTree<LookupWidget>
}

impl FromWorld for SpatialLookup {
    fn from_world(_world: &mut World) -> Self {
        SpatialLookup{rtree: RTree::new()}
    }
}

static TEXT_BOX_SIZE: Vec2 = Vec2::new(100.0, 50.0);
static DEFAULT_WIDGET_SIZE: Vec2 = Vec2::new(100.0, 100.0);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(255, 255, 255)))
        .init_resource::<WorldState>()
        .init_resource::<SpatialLookup>()
        .init_resource::<KamiLookups>()
        .init_resource::<WidgetQueues>()
        .add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_startup_system(world_setup)
        .add_system(cursor_system)
        .add_system(widget_move_system)
        .add_system(unclick_system)
        .add_system(click_system)
        .add_system(spawn_system)
        .add_system(delete_system)
        .run();
}

fn get_rect(transform: &Transform) -> Rect {
    Rect::from_center_size(transform.translation.truncate(), DEFAULT_WIDGET_SIZE)
}

fn widget_move_system(mut widget_query: Query<&mut Transform>,
                      world_state: ResMut<WorldState>,
                ) {
    if let Some((e, _)) = world_state.transforming {
        if let Ok(mut transform) = widget_query.get_mut(e) {
            transform.translation = world_state.cursor_pos.extend(transform.translation.z);
        }
    }
}

fn unclick_system(mut widget_query: Query<(&Transform, &KamiWidget)>,
                mut world_state: ResMut<WorldState>,
                buttons: Res<Input<MouseButton>>,
                kami_lookups: Res<KamiLookups>,
                mut lookup: ResMut<SpatialLookup>,
                mut queues: ResMut<WidgetQueues>
                ) {
    if buttons.just_released(MouseButton::Left) {
        if let Some((e, start_t)) = world_state.transforming {
            lookup.rtree.remove(&LookupWidget{e, priority: start_t.translation.z, bounds: get_rect(&start_t)});
            if let Ok((_, this_kami)) = widget_query.get(e) {
                if let Some(l) = lookup.rtree.locate_all_at_point_mut(&world_state.cursor_pos.into()).reduce(|acc, e| if acc.priority >= e.priority {acc} else {e}) {
                    let other_e = l.e;
                    let (_, other_kami) = widget_query.get(other_e).expect("It's in the rtree, it should exist");
                    let key_guess_1: (&str, &str) = (&this_kami.internal_name, &other_kami.internal_name);
                    let key_guess_2: (&str, &str) = (&other_kami.internal_name, &this_kami.internal_name);

                    let actions_opt = if kami_lookups.combos.contains_key(&key_guess_1) {kami_lookups.combos.get(&key_guess_1)} else {kami_lookups.combos.get(&key_guess_2)};
                   
                    if let Some(actions) = actions_opt {
                        let mut rng = rand::thread_rng();
                        for action in actions {
                            match action {
                                Action::Create(internal_name) => queues.spawn_queue.push((
                                    KamiWidget {internal_name: internal_name},
                                    Vec3::new(rng.gen_range(-100.0..100.0),
                                              rng.gen_range(-100.0..100.0),
                                              rng.gen_range(0.0..500.0))
                                    )),
                                Action::Delete(internal_name) => if &this_kami.internal_name == internal_name
                                    {queues.delete_queue.push(e)} else
                                    {queues.delete_queue.push(other_e)},
                            }
                        }
                    }
                }

            }
            if let Ok((this_transform, _)) = widget_query.get_mut(e) {
                lookup.rtree.insert(LookupWidget{
                    bounds: get_rect(&this_transform),
                    priority: this_transform.translation.z,
                    e,
                });
            }
        }
        world_state.transforming = None;
    }
}

fn click_system(mut widget_query: Query<(&Transform, With<KamiWidget>)>,
                mut world_state: ResMut<WorldState>,
                buttons: ResMut<Input<MouseButton>>,
                mut lookup: ResMut<SpatialLookup>,
                ) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(l) = lookup.rtree.locate_all_at_point_mut(&world_state.cursor_pos.into()).reduce(|acc, e| if acc.priority >= e.priority {acc} else {e}) {
            let e = l.e;
            let (transform, _) = widget_query.get_mut(e).expect("It's in the rtree, it should exist");
            world_state.transforming = Some((e, transform.clone()));
        }
    } 
}

fn spawn_system(mut commands: Commands, kami_lookups: ResMut<KamiLookups>,
                mut space_lookup: ResMut<SpatialLookup>, asset_server: Res<AssetServer>,
                mut queues: ResMut<WidgetQueues>) {
    let queue = queues.spawn_queue.drain(..);
    for (kwidget, pos) in queue {
        /*if kami_lookups.done.get(&kwidget.internal_name) == Some(&true) {
            continue
        };
        kami_lookups.done.insert(&kwidget.internal_name, true);*/
        let full_name = kami_lookups.full_name.get(&kwidget.internal_name).unwrap();
        let asset_name = kami_lookups.asset_name.get(&kwidget.internal_name).unwrap();
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let text_style = TextStyle {
            font,
            font_size: 12.0,
            color: Color::BLACK,
        };
        let text_box_position = Vec2::new(0.0, -DEFAULT_WIDGET_SIZE.y/2.0 - TEXT_BOX_SIZE.y/2.0);
        let text = Text2dBundle {
            text: Text::from_section(full_name.clone(), text_style).with_alignment(TextAlignment::CENTER),
            text_2d_bounds: Text2dBounds {
                // Wrap text in the rectangle
                size: TEXT_BOX_SIZE*DEFAULT_WIDGET_SIZE,
            },
            transform: Transform::from_xyz(
                text_box_position.x,
                text_box_position.y,
                0.1,
            ),
            ..default()
        };
        let transform = Transform::from_translation(pos);
        let spawned_sprite = commands.spawn((
                kwidget,
                SpriteBundle {
                    texture: asset_server.load(asset_name.to_string()),
                    transform: transform.clone(),
                    ..default()
                },
        )).id();
        let spawned_text = commands.spawn(text).id();
        commands.entity(spawned_sprite).push_children(&[spawned_text]);
        space_lookup.rtree.insert(LookupWidget {
            bounds: get_rect(&transform),
            priority: transform.translation.z,
            e: spawned_sprite,
        });
    }
}

fn delete_system(mut commands: Commands,
                 widget_query: Query<&Transform>,
                 mut space_lookup: ResMut<SpatialLookup>,
                 mut queues: ResMut<WidgetQueues>) {
    let queue = queues.delete_queue.drain(..);
    for e in queue {
        if let Ok(transform) = widget_query.get(e) {
            space_lookup.rtree.remove(&LookupWidget{
                bounds: get_rect(&transform),
                priority: transform.translation.z,
                e,
            });
            commands.entity(e).despawn_recursive();
        };
    };
}

fn world_setup(world: &mut World) {
    world.spawn((Camera2dBundle::default(), PanCam {
        grab_buttons: vec![MouseButton::Right], // which buttons should drag the camera
        enabled: true, // when false, controls are disabled. See toggle example.
        zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
        min_scale: 0.5, // prevent the camera from zooming too far in
        max_scale: Some(40.), // prevent the camera from zooming too far out
        min_x: Some(-2000.0),
        min_y: Some(-2000.0),
        max_x: Some(2000.0),
        max_y: Some(2000.0),
    }));

    world.resource_scope(|world, mut queues: Mut<WidgetQueues>| {
        let mut rng = rand::thread_rng();

        for internal_name in STARTING_KAMI {
            queues.spawn_queue.push((
                                    KamiWidget {internal_name: internal_name.clone()},
                                    Vec3::new(rng.gen_range(-100.0..100.0),
                                              rng.gen_range(-100.0..100.0),
                                              rng.gen_range(0.0..500.0))
                                    )
                                   )
        }
    });
}

fn cursor_system(
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<PanCam>>,
    mut world_state: ResMut<WorldState>
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).expect("We have a window")
    } else {
        wnds.get_primary().expect("We have a window")
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        world_state.cursor_pos = world_pos;
        //eprintln!("World coords: {}/{}", world_pos.x, world_pos.y);
    }
}
