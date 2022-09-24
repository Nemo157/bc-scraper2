use ggez::{
    graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect, BLACK, WHITE, Text},
    input::mouse::MouseButton,
    Context,
};
use hecs::{World, Entity};
use std::time::{Duration, Instant};
use num_traits::Float;
use mint::Point2;

use crate::{
    data::{Album, Camera, Dragged, Relationship, UnderMouse, User},
    phys::{Distance, Position, Velocity},
};

const LIGHT_RED: Color = Color::new(1.0, 0.0, 0.0, 0.2);

impl From<Position> for Point2<f32> {
    fn from(pos: Position) -> Self {
        [pos.0.x.raw(), pos.0.y.raw()].into()
    }
}

impl From<&Position> for Point2<f32> {
    fn from(pos: &Position) -> Self {
        [pos.0.x.raw(), pos.0.y.raw()].into()
    }
}

impl From<Point2<f32>> for Position {
    fn from(pos: Point2<f32>) -> Self {
        (pos.x, pos.y).into()
    }
}

fn ensure_meshes(world: &mut World, ctx: &mut Context) {
    let to_add_users = world
        .query_mut::<hecs::Without<Mesh, &User>>()
        .into_iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    let to_add_albums = world
        .query_mut::<hecs::Without<Mesh, &Album>>()
        .into_iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    for entity in to_add_users {
        world
            .insert_one(
                entity,
                Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(-5.0, -5.0, 10.0, 10.0),
                    BLACK,
                )
                .unwrap(),
            )
            .unwrap();
    }

    for entity in to_add_albums {
        world
            .insert_one(
                entity,
                Mesh::new_circle(ctx, DrawMode::fill(), [0.0, 0.0], 5.0, 0.1, BLACK).unwrap(),
            )
            .unwrap();
    }
}

fn transform(world: &mut World, ctx: &mut Context) {
    for (_, pos) in world.query_mut::<hecs::With<Camera, &Position>>() {
        ggez::graphics::set_transform(ctx, DrawParam::new().dest(pos).to_matrix());
        ggez::graphics::apply_transformations(ctx).unwrap();
    }
}

fn draw_entities(world: &mut World, ctx: &mut Context, delta: Duration) {
    for (_, (mesh, pos, vel)) in world.query_mut::<(
        &Mesh,
        &Position,
        Option<hecs::Without<UnderMouse, &Velocity>>,
    )>() {
        let pos = vel.map(|vel| pos + *vel * delta).unwrap_or(*pos);
        ggez::graphics::draw(ctx, mesh, DrawParam::from((pos,))).unwrap();
    }
}

fn draw_relationships(world: &mut World, ctx: &mut Context, delta: Duration) {
    let mut mesh = MeshBuilder::new();
    let mut has_any = false;
    for (_, rel) in &mut world.query::<&Relationship>() {
        let (pos1, vel1, pos2, vel2) = (
            world.get::<Position>(rel.from).unwrap(),
            world.get::<Velocity>(rel.from),
            world.get::<Position>(rel.to).unwrap(),
            world.get::<Velocity>(rel.to),
        );
        let pos1 = vel1.map(|vel1| *pos1 + *vel1 * delta).unwrap_or(*pos1);
        let pos2 = vel2.map(|vel2| *pos2 + *vel2 * delta).unwrap_or(*pos2);
        let dist = pos1 - pos2;
        if dist.chebyshev().abs() > 1.0 {
            mesh.line(
                &[pos1, pos2],
                0.5,
                LIGHT_RED,
            )
            .unwrap();
            has_any = true;
        }
    }
    if has_any {
        let mesh = mesh.build(ctx).unwrap();
        ggez::graphics::draw(ctx, &mesh, DrawParam::default()).unwrap();
    }
}

fn draw_status_bar(world: &mut World, ctx: &mut Context) {
    for (_, album) in world.query_mut::<hecs::With<UnderMouse, &Album>>() {
        ggez::graphics::draw(ctx, Text::new("album: ").add(album.url.as_str()), DrawParam::from(([0.0, 0.0], BLACK))).unwrap();
    }
    for (_, user) in world.query_mut::<hecs::With<UnderMouse, &User>>() {
        ggez::graphics::draw(ctx, Text::new("user: ").add(user.url.as_str()), DrawParam::from(([0.0, 0.0], BLACK))).unwrap();
    }
}

pub fn draw(world: &mut World, ctx: &mut Context, delta: Duration) {
    ggez::graphics::clear(ctx, WHITE);
    ensure_meshes(world, ctx);
    ggez::graphics::origin(ctx);
    ggez::graphics::apply_transformations(ctx).unwrap();
    draw_status_bar(world, ctx);
    transform(world, ctx);
    draw_entities(world, ctx, delta);
    draw_relationships(world, ctx, delta);
}

fn update_drag(world: &mut World, mouse_pos: Position, delta: Distance) {
    let mut dragged_item = false;

    for (_, pos) in world.query_mut::<hecs::With<Dragged, &mut Position>>() {
        *pos = mouse_pos;
        dragged_item = true;
    }

    if !dragged_item {
        for (_, pos) in world.query_mut::<hecs::With<Camera, &mut Position>>() {
            *pos += delta;
        }
    }
}

fn start_drag(world: &mut World) {
    let to_add = world
        .query_mut::<hecs::With<UnderMouse, &Position>>()
        .into_iter()
        .map(|(entity, &pos)| (entity, pos))
        .collect::<Vec<_>>();
    for (entity, pos) in to_add {
        world.insert_one(entity, Dragged(pos, Instant::now())).unwrap();
    }
}

fn stop_drag(world: &mut World) -> Option<Entity> {
    static CLICK_DURATION: Duration = Duration::from_millis(100);
    let to_remove = world
        .query_mut::<hecs::With<Dragged, &Position>>()
        .into_iter()
        .map(|(entity, &pos)| (entity, pos))
        .collect::<Vec<_>>();
    let mut clicked = None;
    for (entity, pos) in to_remove {
        let Dragged(start_pos, start_time) = world.remove_one::<Dragged>(entity).unwrap();
        if (start_pos - pos).chebyshev() < 5.0 && start_time.elapsed() < CLICK_DURATION {
            clicked = Some(entity);
        }
    }
    clicked
}

fn offset_to_camera(world: &mut World, mut mouse_pos: Position) -> Position {
    for (_, pos) in world.query_mut::<hecs::With<Camera, &mut Position>>() {
        mouse_pos.0 -= pos.0;
    }
    mouse_pos
}

fn update_under_mouse(world: &mut World, mouse_pos: Position) {
    let to_remove = world
        .query_mut::<hecs::With<UnderMouse, &Position>>()
        .into_iter()
        .filter(|(_, &pos)| {
            (pos - mouse_pos).chebyshev() > 5.0
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    let to_add = world
        .query_mut::<hecs::Without<UnderMouse, &Position>>()
        .into_iter()
        .filter(|(_, &pos)| {
            (pos - mouse_pos).chebyshev() < 5.0
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    for entity in to_remove {
        world.remove_one::<UnderMouse>(entity).unwrap();
    }

    for entity in to_add {
        world.insert_one(entity, UnderMouse).unwrap();
    }
}

pub fn mouse_down(world: &mut World, ctx: &mut Context, button: MouseButton, pos: Position) {
    let _ = (ctx, pos);
    if button == MouseButton::Left {
        start_drag(world);
    }
}

pub fn mouse_up(world: &mut World, ctx: &mut Context, button: MouseButton, pos: Position) -> Option<Entity> {
    let _ = (ctx, pos);
    if button == MouseButton::Left {
        stop_drag(world)
    } else {
        None
    }
}

pub fn mouse_motion(world: &mut World, ctx: &mut Context, pos: Position, delta: Distance) {
    let pos = offset_to_camera(world, pos);
    if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
        update_drag(world, pos, delta);
    }
}

pub fn update(world: &mut World, ctx: &mut Context) {
    let mouse_pos = ggez::input::mouse::position(ctx);
    let mouse_pos = offset_to_camera(world, Position::from(mouse_pos));
    update_under_mouse(world, mouse_pos);
}

pub fn init(world: &mut World, _: &mut Context) {
    world.spawn((Camera, Position::new(0.0, 0.0)));
}
