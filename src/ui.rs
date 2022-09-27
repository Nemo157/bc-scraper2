use ggez::{
    graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect, BLACK, WHITE, Text},
    input::mouse::MouseButton,
    Context,
};
use hecs::{World, Entity};
use std::time::{Duration, Instant};

use phys::{Distance, Position, Velocity, Float};
use data::{Album, Camera, Dragged, Relationship, UnderMouse, User, Zoom};

const LIGHT_RED: Color = Color::new(1.0, 0.0, 0.0, 0.2);
const MODE: once_cell::sync::Lazy::<dark_light::Mode> = once_cell::sync::Lazy::new(|| dark_light::detect());

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
                    match *MODE { dark_light::Mode::Light => BLACK, dark_light::Mode::Dark => WHITE },
                )
                .unwrap(),
            )
            .unwrap();
    }

    for entity in to_add_albums {
        world
            .insert_one(
                entity,
                Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    [0.0, 0.0],
                    5.0,
                    0.1,
                    match *MODE { dark_light::Mode::Light => BLACK, dark_light::Mode::Dark => WHITE },
                )
                .unwrap(),
            )
            .unwrap();
    }
}

fn transform(world: &mut World, ctx: &mut Context) {
    for (_, (pos, zoom)) in world.query_mut::<hecs::With<Camera, (&Position, &Zoom)>>() {
        ggez::graphics::set_transform(ctx, DrawParam::new().dest(pos).scale([zoom.0, zoom.0]).to_matrix());
        ggez::graphics::apply_transformations(ctx).unwrap();
    }
}

fn draw_entities(world: &mut World, ctx: &mut Context, delta: Duration, (tl, br): (Position, Position)) -> usize {
    let mut count = 0;
    for (_, (mesh, pos, vel)) in world.query_mut::<(
        &Mesh,
        &Position,
        Option<hecs::Without<UnderMouse, &Velocity>>,
    )>() {
        let pos = vel.map(|vel| pos + *vel * delta).unwrap_or(*pos);
        if pos > tl && pos < br {
            ggez::graphics::draw(ctx, mesh, DrawParam::from((pos,))).unwrap();
            count += 1;
        }
    }
    count
}

fn draw_relationships(world: &mut World, ctx: &mut Context, delta: Duration, (tl, br): (Position, Position)) -> usize {
    let mut mesh = MeshBuilder::new();
    let mut count = 0;
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
        if dist.chebyshev().abs() > 1.0 && ((pos1 > tl && pos1 < br) || (pos2 > tl && pos2 < br)) {
            mesh.line(
                &[pos1, pos2],
                0.5,
                LIGHT_RED,
            )
            .unwrap();
            count += 1;
        }
    }
    if count > 0 {
        let mesh = mesh.build(ctx).unwrap();
        ggez::graphics::draw(ctx, &mesh, DrawParam::default()).unwrap();
    }
    count
}

fn draw_status_bar(world: &mut World, ctx: &mut Context, tps: f64, fps: f64, nodes: usize, lines: usize) {
    let albums = world.query_mut::<hecs::With<Album, ()>>().into_iter().len();
    let users = world.query_mut::<hecs::With<User, ()>>().into_iter().len();
    let links = world.query_mut::<hecs::With<Relationship, ()>>().into_iter().len();

    let mut text = Text::new(format!(indoc::indoc!("
        tps: {:.2}
        fps: {:.2}
        albums, users, links: {} {} {}
        drawn: {}/{} {}/{}
    "), tps, fps, albums, users, links, nodes, (albums + users), lines, links));

    for (_, album) in world.query_mut::<hecs::With<UnderMouse, &Album>>() {
        let url = &album.url;
        text.add(format!("\nalbum: {url}"));
    }

    for (_, user) in world.query_mut::<hecs::With<UnderMouse, &User>>() {
        let url = &user.url;
        text.add(format!("\nuser: {url}"));
    }

    ggez::graphics::draw(
        ctx,
        &text,
        DrawParam::from((
            [0.0, 0.0],
            match *MODE {
                dark_light::Mode::Light => BLACK,
                dark_light::Mode::Dark => WHITE
            }
        )),
    ).unwrap();
}

pub fn draw(world: &mut World, ctx: &mut Context, delta: Duration, tps: f64, fps: f64) {
    ggez::graphics::clear(ctx,
            match *MODE {
                dark_light::Mode::Light => WHITE,
                dark_light::Mode::Dark => BLACK,
            });
    ensure_meshes(world, ctx);
    transform(world, ctx);
    let coords = ggez::graphics::screen_coordinates(ctx);
    let (tl, br) = (offset_to_camera(world, Position::new(coords.x, coords.y)), offset_to_camera(world, Position::new(coords.x + coords.w, coords.y + coords.h)));
    let nodes = draw_entities(world, ctx, delta, (tl, br));
    let lines = draw_relationships(world, ctx, delta, (tl, br));
    ggez::graphics::origin(ctx);
    ggez::graphics::apply_transformations(ctx).unwrap();
    draw_status_bar(world, ctx, tps, fps, nodes, lines);
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
    for (_, (pos, zoom)) in world.query_mut::<hecs::With<Camera, (&Position, &Zoom)>>() {
        mouse_pos.0 -= pos.0;
        mouse_pos.0 /= zoom.0;
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

pub fn mouse_wheel(world: &mut World, _ctx: &mut Context, mouse_pos: Position, wheel_vel: Velocity) {
    if wheel_vel.0.y != 0.0 {
        for (_, (pos, zoom)) in world.query_mut::<hecs::With<Camera, (&mut Position, &mut Zoom)>>() {
            let zoom_ratio = if wheel_vel.0.y > 0.0 { 1.5 } else { 1.0 / 1.5 };
            zoom.0 *= zoom_ratio;
            *pos = mouse_pos + (*pos - mouse_pos) * zoom_ratio;
        }
    }
}

pub fn mouse_motion(world: &mut World, ctx: &mut Context, pos: Position, delta: Distance) {
    let pos = offset_to_camera(world, pos);
    if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
        update_drag(world, pos, delta);
    }
}

pub fn resize(world: &mut World, ctx: &mut Context, width: f32, height: f32) {
    let coords = ggez::graphics::screen_coordinates(ctx);
    for (_, pos) in world.query_mut::<hecs::With<Camera, &mut Position>>() {
        *pos += Distance::new((width - coords.w) / 2.0, (height - coords.h) / 2.0);
    }
    ggez::graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, width, height)).unwrap();
}

pub fn update(world: &mut World, ctx: &mut Context) {
    let mouse_pos = ggez::input::mouse::position(ctx);
    let mouse_pos = offset_to_camera(world, Position::from(mouse_pos));
    update_under_mouse(world, mouse_pos);
}

pub fn init(world: &mut World, _: &mut Context) {
    world.spawn((Camera, Position::new(0.0, 0.0), Zoom(1.0)));
}
