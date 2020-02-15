use hecs::World;
use std::time::Duration;
use ggez::{Context, input::mouse::MouseButton, graphics::{Rect, Mesh, DrawMode, DrawParam, WHITE, BLACK, Color, MeshBuilder}};

use crate::{
    phys::{Position, Velocity, Distance},
    data::{Relationship, UnderMouse, User, Album, Camera},
};

const LIGHT_RED: Color = Color::new(1.0, 0.0, 0.0, 0.2);

fn ensure_meshes(world: &mut World, ctx: &mut Context) {
    let to_add_users = world.query::<hecs::Without<Mesh, &User>>().iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    let to_add_albums = world.query::<hecs::Without<Mesh, &Album>>().iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    for entity in to_add_users {
        world.insert_one(
            entity,
            Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(-5.0, -5.0, 10.0, 10.0), BLACK).unwrap(),
        ).unwrap();
    }

    for entity in to_add_albums {
        world.insert_one(
            entity,
            Mesh::new_circle(ctx, DrawMode::fill(), [0.0, 0.0], 5.0, 0.1, BLACK).unwrap(),
        ).unwrap();
    }
}

fn transform(world: &mut World, ctx: &mut Context) {
    for (_, pos) in &mut world.query::<hecs::With<Camera, &Position>>() {
        ggez::graphics::set_transform(ctx, DrawParam::new().dest([pos.0.x, pos.0.y]).to_matrix());
        ggez::graphics::apply_transformations(ctx).unwrap();
    }
}

fn draw_entities(world: &mut World, ctx: &mut Context, delta: Duration) {
    for (_, (mesh, pos, vel)) in &mut world.query::<(&Mesh, &Position, Option<hecs::Without<UnderMouse, &Velocity>>)>() {
        let pos = vel.map(|vel| pos + *vel * delta).unwrap_or(*pos);
        ggez::graphics::draw(ctx, mesh, DrawParam::from(([pos.0.x, pos.0.y],))).unwrap();
    }
}

fn draw_relationships(world: &mut World, ctx: &mut Context, delta: Duration) {
    let mut mesh = MeshBuilder::new();
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
        if dist.0.x.abs() > 1.0 && dist.0.y.abs() > 1.0 {
            mesh.line(&[[pos1.0.x, pos1.0.y], [pos2.0.x, pos2.0.y]], 0.5, LIGHT_RED).unwrap();
        }
    }
    let mesh = mesh.build(ctx).unwrap();
    ggez::graphics::draw(ctx, &mesh, DrawParam::default()).unwrap();
}

pub fn draw(world: &mut World, ctx: &mut Context, delta: Duration) {
    ggez::graphics::clear(ctx, WHITE);
    ensure_meshes(world, ctx);
    transform(world, ctx);
    draw_entities(world, ctx, delta);
    draw_relationships(world, ctx, delta);
}

fn update_drag(world: &mut World, mouse_pos: Position, delta: Distance) {
    let mut dragged_item = false;

    for (_, pos) in &mut world.query::<hecs::With<UnderMouse, &mut Position>>() {
        *pos = mouse_pos;
        dragged_item = true;
    }

    if !dragged_item {
        for (_, pos) in &mut world.query::<hecs::With<Camera, &mut Position>>() {
            *pos += delta;
        }
    }
}

fn offset_to_camera(world: &mut World, mut mouse_pos: Position) -> Position {
    for (_, pos) in &mut world.query::<hecs::With<Camera, &mut Position>>() {
        mouse_pos.0 -= pos.0;
    }
    mouse_pos
}

fn update_under_mouse(world: &mut World, mouse_pos: Position) {
    let to_remove = world.query::<hecs::With<UnderMouse, &Position>>().iter()
        .filter(|(_, pos)| {
            let dist = *pos - mouse_pos;
            dist.0.x.abs() > 5.0 || dist.0.y.abs() > 5.0
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    let to_add = world.query::<hecs::Without<UnderMouse, &Position>>().iter()
        .filter(|(_, pos)| {
            let dist = *pos - mouse_pos;
            dist.0.x.abs() < 5.0 && dist.0.y.abs() < 5.0
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

pub fn mouse_motion(world: &mut World, ctx: &mut Context, pos: Position, delta: Distance) {
    let pos = offset_to_camera(world, pos);
    if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
        update_drag(world, pos, delta);
    }
}

pub fn update(world: &mut World, ctx: &mut Context) {
    let mouse_pos = ggez::input::mouse::position(ctx);
    let mouse_pos: [f32; 2] = mouse_pos.into();
    let mouse_pos = offset_to_camera(world, Position::from(mouse_pos));
    update_under_mouse(world, mouse_pos);
}

pub fn init(world: &mut World, _: &mut Context) {
    world.spawn((Camera, Position::new(0.0, 0.0)));
}
