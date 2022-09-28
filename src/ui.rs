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
static MODE: once_cell::sync::Lazy::<dark_light::Mode> = once_cell::sync::Lazy::new(dark_light::detect);

fn user_mesh(ctx: &mut Context) -> &'static Mesh {
    static MESH: once_cell::sync::OnceCell::<Mesh> = once_cell::sync::OnceCell::new();
    MESH.get_or_init(|| {
        Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(-5.0, -5.0, 10.0, 10.0),
            match *MODE { dark_light::Mode::Light => BLACK, dark_light::Mode::Dark => WHITE },
        )
        .unwrap()
    })
}

fn album_mesh(ctx: &mut Context) -> &'static Mesh {
    static MESH: once_cell::sync::OnceCell::<Mesh> = once_cell::sync::OnceCell::new();
    MESH.get_or_init(|| {
        Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            [0.0, 0.0],
            5.0,
            0.1,
            match *MODE { dark_light::Mode::Light => BLACK, dark_light::Mode::Dark => WHITE },
        )
        .unwrap()
    })
}

fn entity_mesh(ctx: &mut Context, entity: &Entity) -> &'static Mesh {
    match entity.data {
        EntityData::Album(_) => album_mesh(ctx),
        EntityData::User(_) => user_mesh(ctx),
    }
}

#[derive(Debug)]
struct Camera {
    position: Position,
    zoom: f32,
}

#[derive(Debug)]
pub struct Ui {
    camera: Camera,
    pub enable_lines: bool,
    pub enable_nodes: bool,
}

impl Ui {
    pub fn new() -> Self {
        Self { 
            camera: Camera {
                position: Position::new(0.0, 0.0),
                zoom: 1.0,
            },
            enable_lines: true,
            enable_nodes: true,
        }
    }

    fn draw_entities(&self, data: &Data, ctx: &mut Context, delta: Duration, (tl, br): (Position, Position)) -> usize {
        let mut count = 0;
        for entity in &data.entities {
            let pos = entity.position + entity.velocity * delta;
            if pos > tl && pos < br {
                let mesh = entity_mesh(ctx, entity);
                ggez::graphics::draw(ctx, mesh, DrawParam::from((pos,))).unwrap();
                count += 1;
            }
        }
        count
    }

    fn draw_relationships(&self, data: &Data, ctx: &mut Context, delta: Duration) -> usize {
        let mut mesh = MeshBuilder::new();
        let mut count = 0;
        for rel in &data.relationships {
            let entity1 = &data.entities[rel.album];
            let entity2 = &data.entities[rel.user];
            let pos1 = entity1.position + entity1.velocity * delta;
            let pos2 = entity2.position + entity2.velocity * delta;
            let dist = pos1 - pos2;
            if dist.chebyshev().abs() > 1.0 {
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

    fn draw_status_bar(&self, data: &Data, ctx: &mut Context, tps: f64, sim_duration: Duration, fps: f64, frame_duration: Duration, nodes: usize, _lines: usize) {
        let albums = data.albums.len();
        let users = data.users.len();
        let links = data.relationships.len();

        let mut text = Text::new(format!(indoc::indoc!("
            tps: {:.2} ({:.2?})
            fps: {:.2} ({:.2?})
            albums: {}
            users: {}
            links: {}
            drawn: {}/{}
        "), tps, sim_duration, fps, frame_duration, albums, users, links, nodes, (albums + users)));

        for entity in &data.entities {
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

    pub fn draw(&mut self, data: &Data, ctx: &mut Context, delta: Duration, tps: f64, sim_duration: Duration, fps: f64, frame_duration: Duration) {
        ggez::graphics::clear(ctx, match *MODE { dark_light::Mode::Light => WHITE, dark_light::Mode::Dark => BLACK });
        ggez::graphics::set_transform(ctx, DrawParam::new().dest(self.camera.position).scale([self.camera.zoom, self.camera.zoom]).to_matrix());
        ggez::graphics::apply_transformations(ctx).unwrap();
        let coords = ggez::graphics::screen_coordinates(ctx);
        let (tl, br) = (self.offset_to_camera(Position::new(coords.x, coords.y)), self.offset_to_camera(Position::new(coords.x + coords.w, coords.y + coords.h)));
        let lines = self.enable_lines.then(|| self.draw_relationships(data, ctx, delta)).unwrap_or_default();
        let nodes = self.enable_nodes.then(|| self.draw_entities(data, ctx, delta, (tl, br))).unwrap_or_default();
        ggez::graphics::origin(ctx);
        ggez::graphics::apply_transformations(ctx).unwrap();
        self.draw_status_bar(data, ctx, tps, sim_duration, fps, frame_duration, nodes, lines);
    }

    fn update_drag(&mut self, data: &mut Data, mouse_pos: Position, delta: Distance) {
        let mut dragged_item = false;

        for entity in &mut data.entities {
            if entity.dragged.is_some() {
                entity.position = mouse_pos;
                dragged_item = true;
            }
        }

        if !dragged_item {
            self.camera.position += delta;
        }
    }

    fn start_drag(&mut self, data: &mut Data) {
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

    fn stop_drag<'a>(&mut self, data: &'a mut Data) -> Option<&'a Entity> {
        static CLICK_DURATION: Duration = Duration::from_millis(100);

        for entity in &mut data.entities {
            if let Some(Drag { start_position, start_time }) = entity.dragged.take() {
                if (start_position - entity.position).chebyshev() < 5.0 && start_time.elapsed() < CLICK_DURATION {
                    return Some(entity);
                }
            }
        }

        None
    }

    fn offset_to_camera(&self, position: Position) -> Position {
        Position::from((position.0 - self.camera.position.0) / self.camera.zoom)
    }

    fn update_under_mouse(&mut self, data: &mut Data, mouse_pos: Position) {
        for entity in &mut data.entities {
            entity.is_under_mouse = (entity.position - mouse_pos).chebyshev() < 5.0;
        }
    }

    pub fn mouse_down(&mut self, data: &mut Data, ctx: &mut Context, button: MouseButton, pos: Position) {
        let _ = (ctx, pos);
        if button == MouseButton::Left {
            self.start_drag(data);
        }
    }

    pub fn mouse_up<'a>(&mut self, data: &'a mut Data, ctx: &mut Context, button: MouseButton, pos: Position) -> Option<&'a Entity> {
        let _ = (ctx, pos);
        if button == MouseButton::Left {
            self.stop_drag(data)
        } else {
            None
        }
    }

    pub fn mouse_wheel(&mut self, mouse_pos: Position, wheel_vel: Velocity) {
        if wheel_vel.0.y != 0.0 {
            let zoom_ratio = if wheel_vel.0.y > 0.0 { 1.5 } else { 1.0 / 1.5 };
            self.camera.zoom *= zoom_ratio;
            self.camera.position = mouse_pos + (self.camera.position - mouse_pos) * zoom_ratio;
        }
    }

    pub fn mouse_motion(&mut self, data: &mut Data, ctx: &mut Context, pos: Position, delta: Distance) {
        let pos = self.offset_to_camera(pos);
        if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
            self.update_drag(data, pos, delta);
        }
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let coords = ggez::graphics::screen_coordinates(ctx);
        self.camera.position += Distance::new((width - coords.w) / 2.0, (height - coords.h) / 2.0);
        ggez::graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, width, height)).unwrap();
    }

    pub fn update(&mut self, data: &mut Data, ctx: &mut Context) {
        let mouse_pos = ggez::input::mouse::position(ctx);
        let mouse_pos = self.offset_to_camera(Position::from(mouse_pos));
        self.update_under_mouse(data, mouse_pos);
    }
}
