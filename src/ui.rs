use ggez::{
    graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect, BLACK, WHITE, Text},
    input::mouse::MouseButton,
    Context,
};
use std::time::{Duration, Instant};

use opt::{
    phys::{Distance, Position, Velocity, Float},
    data::{Data, Album, User, Entity, EntityData, Drag},
};

const LIGHT_RED: Color = Color::new(1.0, 0.0, 0.0, 0.2);
const MODE: once_cell::sync::Lazy::<dark_light::Mode> = once_cell::sync::Lazy::new(|| dark_light::detect());

fn user_mesh(ctx: &mut Context) -> Mesh {
    Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(-5.0, -5.0, 10.0, 10.0),
        match *MODE { dark_light::Mode::Light => BLACK, dark_light::Mode::Dark => WHITE },
    )
    .unwrap()
}

fn album_mesh(ctx: &mut Context) -> Mesh {
    Mesh::new_circle(
        ctx,
        DrawMode::fill(),
        [0.0, 0.0],
        5.0,
        0.1,
        match *MODE { dark_light::Mode::Light => BLACK, dark_light::Mode::Dark => WHITE },
    )
    .unwrap()
}

fn entity_mesh(ctx: &mut Context, entity: &Entity) -> Mesh {
    match entity.data {
        EntityData::Album(_) => album_mesh(ctx),
        EntityData::User(_) => user_mesh(ctx),
    }
}

fn transform(data: &Data, ctx: &mut Context) {
    ggez::graphics::set_transform(ctx, DrawParam::new().dest(data.camera.position).scale([data.camera.zoom, data.camera.zoom]).to_matrix());
    ggez::graphics::apply_transformations(ctx).unwrap();
}

fn draw_entities(data: &mut Data, ctx: &mut Context, delta: Duration, (tl, br): (Position, Position)) -> usize {
    let mut count = 0;
    for entity in &data.entities {
        let pos = entity.position + entity.velocity * delta;
        if pos > tl && pos < br {
            let mesh = entity_mesh(ctx, entity);
            ggez::graphics::draw(ctx, &mesh, DrawParam::from((pos,))).unwrap();
            count += 1;
        }
    }
    count
}

fn draw_relationships(data: &mut Data, ctx: &mut Context, delta: Duration, (tl, br): (Position, Position)) -> usize {
    let mut mesh = MeshBuilder::new();
    let mut count = 0;
    for rel in &data.relationships {
        let entity1 = &data.entities[rel.album];
        let entity2 = &data.entities[rel.user];
        let pos1 = entity1.position + entity1.velocity * delta;
        let pos2 = entity2.position + entity2.velocity * delta;
        let dist = pos1 - pos2;
        if dist.chebyshev().abs() > 1.0 && ((pos1 > tl && pos1 < br) || (pos2 > tl && pos2 < br)) {
            mesh.line(&[pos1, pos2], 0.5, LIGHT_RED).unwrap();
            count += 1;
        }
    }
    if count > 0 {
        let mesh = mesh.build(ctx).unwrap();
        ggez::graphics::draw(ctx, &mesh, DrawParam::default()).unwrap();
    }
    count
}

fn draw_status_bar(data: &mut Data, ctx: &mut Context, tps: f64, fps: f64, nodes: usize, lines: usize) {
    let albums = data.albums.len();
    let users = data.users.len();
    let links = data.relationships.len();

    let mut text = Text::new(format!(indoc::indoc!("
        tps: {:.2}
        fps: {:.2}
        albums, users, links: {} {} {}
        drawn: {}/{} {}/{}
    "), tps, fps, albums, users, links, nodes, (albums + users), lines, links));

    for entity in &mut data.entities {
        if entity.is_under_mouse {
            match &entity.data {
                EntityData::Album(Album { url, .. }) => {
                    text.add(format!("\nalbum: {url}"));
                }
                EntityData::User(User { url, .. }) => {
                    text.add(format!("\nuser: {url}"));
                }
            }
        }
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

pub fn draw(data: &mut Data, ctx: &mut Context, delta: Duration, tps: f64, fps: f64) {
    ggez::graphics::clear(ctx,
            match *MODE {
                dark_light::Mode::Light => WHITE,
                dark_light::Mode::Dark => BLACK,
            });
    transform(data, ctx);
    let coords = ggez::graphics::screen_coordinates(ctx);
    let (tl, br) = (offset_to_camera(data, Position::new(coords.x, coords.y)), offset_to_camera(data, Position::new(coords.x + coords.w, coords.y + coords.h)));
    let lines = draw_relationships(data, ctx, delta, (tl, br));
    let nodes = draw_entities(data, ctx, delta, (tl, br));
    ggez::graphics::origin(ctx);
    ggez::graphics::apply_transformations(ctx).unwrap();
    draw_status_bar(data, ctx, tps, fps, nodes, lines);
}

fn update_drag(data: &mut Data, mouse_pos: Position, delta: Distance) {
    let mut dragged_item = false;

    for entity in &mut data.entities {
        if entity.dragged.is_some() {
            entity.position = mouse_pos;
            dragged_item = true;
        }
    }

    if !dragged_item {
        data.camera.position += delta;
    }
}

fn start_drag(data: &mut Data) {
    for entity in &mut data.entities {
        if entity.is_under_mouse {
            entity.dragged = Some(Drag {
                start_position: entity.position,
                start_time: Instant::now(),
            });
            return;
        }
    }
}

fn stop_drag(data: &mut Data) -> Option<&Entity> {
    static CLICK_DURATION: Duration = Duration::from_millis(100);

    for entity in &mut data.entities {
        if let Some(Drag { start_position, start_time }) = entity.dragged.take() {
            if (start_position - entity.position).chebyshev() < 5.0 && start_time.elapsed() < CLICK_DURATION {
                return Some(entity);
            }
        }
    }

    return None;
}

fn offset_to_camera(data: &mut Data, mouse_pos: Position) -> Position {
    Position::from((mouse_pos.0 - data.camera.position.0) / data.camera.zoom)
}

fn update_under_mouse(data: &mut Data, mouse_pos: Position) {
    for entity in &mut data.entities {
        let dist = (entity.position - mouse_pos).chebyshev();
        if dist > 5.0 {
            entity.is_under_mouse = false;
        } else if dist < 5.0 {
            entity.is_under_mouse = true;
        }
    }
}

pub fn mouse_down(data: &mut Data, ctx: &mut Context, button: MouseButton, pos: Position) {
    let _ = (ctx, pos);
    if button == MouseButton::Left {
        start_drag(data);
    }
}

pub fn mouse_up<'a>(data: &'a mut Data, ctx: &mut Context, button: MouseButton, pos: Position) -> Option<&'a Entity> {
    let _ = (ctx, pos);
    if button == MouseButton::Left {
        stop_drag(data)
    } else {
        None
    }
}

pub fn mouse_wheel(data: &mut Data, _ctx: &mut Context, mouse_pos: Position, wheel_vel: Velocity) {
    if wheel_vel.0.y != 0.0 {
        let zoom_ratio = if wheel_vel.0.y > 0.0 { 1.5 } else { 1.0 / 1.5 };
        data.camera.zoom *= zoom_ratio;
        data.camera.position = mouse_pos + (data.camera.position - mouse_pos) * zoom_ratio;
    }
}

pub fn mouse_motion(data: &mut Data, ctx: &mut Context, pos: Position, delta: Distance) {
    let pos = offset_to_camera(data, pos);
    if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
        update_drag(data, pos, delta);
    }
}

pub fn resize(data: &mut Data, ctx: &mut Context, width: f32, height: f32) {
    let coords = ggez::graphics::screen_coordinates(ctx);
    data.camera.position += Distance::new((width - coords.w) / 2.0, (height - coords.h) / 2.0);
    ggez::graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, width, height)).unwrap();
}

pub fn update(data: &mut Data, ctx: &mut Context) {
    let mouse_pos = ggez::input::mouse::position(ctx);
    let mouse_pos = offset_to_camera(data, Position::from(mouse_pos));
    update_under_mouse(data, mouse_pos);
}

pub fn init(data: &mut Data, _: &mut Context) {
    data.camera.zoom = 1.0;
}
