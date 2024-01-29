use ggez::{
    graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect, Text, Canvas},
    input::mouse::MouseButton,
    Context,
};
use std::{time::{Duration, Instant}, collections::BTreeMap};
use itertools::Itertools;

use opt::{
    phys::{Distance, Position, Velocity, Float},
    data::{Data, Album, User, Entity, EntityData, Drag},
};

const LIGHT_RED: Color = Color::new(1.0, 0.0, 0.0, 0.2);

#[derive(Debug)]
struct Camera {
    position: Position,
    zoom: f32,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
enum EntityTag {
    Album,
    User
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
struct MeshKey {
    tag: EntityTag,
    is_scraped: bool,
    is_under_mouse: bool,
}

#[derive(Debug)]
pub struct Ui {
    camera: Camera,
    pub enable_lines: bool,
    pub enable_nodes: bool,
    meshes: BTreeMap<MeshKey, Mesh>,
    foreground: Color,
    background: Color,
    width: f32,
    height: f32,
}

impl Ui {
    pub fn new(ctx: &mut Context) -> Self {
        let mode = dark_light::detect();
        let (fg, bg) = match mode {
            dark_light::Mode::Light | dark_light::Mode::Default => (Color::BLACK, Color::WHITE),
            dark_light::Mode::Dark => (Color::WHITE, Color::BLACK),
        };

        let highlight = Color::new(0.2, 1.0, 0.2, 1.0);
        let scraped = Color::new(0.2, 0.2, 1.0, 1.0);
        let both = Color::new(0.2, 1.0, 1.0, 1.0);

        let meshes = BTreeMap::from_iter(
            [
                (
                    EntityTag::User,
                    (|ctx, color| Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(-5.0, -5.0, 10.0, 10.0), color).unwrap()) as fn(&mut Context, Color) -> Mesh,
                ),
                (
                    EntityTag::Album,
                    |ctx, color| Mesh::new_circle(ctx, DrawMode::fill(), [0.0, 0.0], 5.0, 0.1, color).unwrap(),
                ),
            ]
                .into_iter()
                .cartesian_product([
                    (false, false, fg),
                    (false, true, highlight),
                    (true, false, scraped),
                    (true, true, both),
                ])
                .map(|((tag, make), (is_scraped, is_under_mouse, color))| {
                    (MeshKey { tag, is_scraped, is_under_mouse }, make(ctx, color))
                }));

        Self { 
            camera: Camera {
                position: Position::new(0.0, 0.0),
                zoom: 1.0,
            },
            enable_lines: true,
            enable_nodes: true,
            meshes,
            foreground: fg,
            background: bg,
            width: 0.0,
            height: 0.0,
        }
    }

    fn mesh_for(&self, entity: &Entity) -> &Mesh {
        let tag = match &*entity.data {
            EntityData::Album(_) => EntityTag::Album,
            EntityData::User(_) => EntityTag::User,
        };
        &self.meshes[&MeshKey { tag, is_scraped: entity.is_scraped, is_under_mouse: entity.is_under_mouse }]
    }

    fn draw_entities(&self, data: &Data, canvas: &mut Canvas, delta: Duration, (tl, br): (Position, Position)) -> usize {
        let mut count = 0;
        for entity in &data.entities {
            let pos = entity.position + entity.velocity * delta;
            if pos > tl && pos < br {
                canvas.draw(self.mesh_for(entity), DrawParam::from(pos));
                count += 1;
            }
        }
        count
    }

    fn draw_relationships(&self, data: &Data, ctx: &mut Context, canvas: &mut Canvas, delta: Duration) -> usize {
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
            let mesh = Mesh::from_data(ctx, mesh.build());
            canvas.draw(&mesh, DrawParam::default());
        }
        count
    }

    fn draw_status_bar(&self, data: &Data, ctx: &mut Context, canvas: &mut Canvas, tps: f64, sim_duration: Duration, fps: f64, frame_duration: Duration, nodes: usize, _lines: usize) {
        let albums = data.albums.len();
        let users = data.users.len();

        let text = Text::new(format!(indoc::indoc!("
            tps: {:.2} ({:.2?})
            fps: {:.2} ({:.2?})
            drawn: {}/{}
        "), tps, sim_duration, fps, frame_duration, nodes, (albums + users)));

        let width = text.measure(ctx).unwrap().x;
        canvas.draw(&text, DrawParam::from([self.width - width as f32, 0.0]).color(self.foreground));

        let links = data.relationships.len();

        let mut text = Text::new(format!(indoc::indoc!("
            albums: {}
            users: {}
            links: {}
        "), albums, users, links));

        for entity in &data.entities {
            if entity.is_under_mouse {
                match &*entity.data {
                    EntityData::Album(Album { url, .. }) => {
                        text.add(format!("\nalbum: {url}"));
                    }
                    EntityData::User(User { url, .. }) => {
                        text.add(format!("\nuser: {url}"));
                    }
                }
            }
        }

        canvas.draw(&text, DrawParam::from([0.0, 0.0]).color(self.foreground));

        let mouse_pos = self.offset_to_camera(Position::from(ctx.mouse.position()));

        let text = Text::new(format!("{}, {}", mouse_pos.0.x, mouse_pos.0.y));
        let height = text.measure(ctx).unwrap().y;
        canvas.draw(&text, DrawParam::from([0.0, self.height - height as f32]).color(self.foreground));
    }

    pub fn draw(&mut self, data: &Data, ctx: &mut Context, delta: Duration, tps: f64, sim_duration: Duration, fps: f64, frame_duration: Duration) {
        let mut canvas = Canvas::from_frame(ctx, self.background);
        canvas.set_projection(DrawParam::new().dest(self.camera.position).scale([self.camera.zoom, self.camera.zoom]).transform.to_bare_matrix());
        let (tl, br) = (self.offset_to_camera(Position::new(0.0, 0.0)), self.offset_to_camera(Position::new(self.width, self.height)));
        let lines = self.enable_lines.then(|| self.draw_relationships(data, ctx, &mut canvas, delta)).unwrap_or_default();
        let nodes = self.enable_nodes.then(|| self.draw_entities(data, &mut canvas, delta, (tl, br))).unwrap_or_default();
        canvas.set_projection(DrawParam::new().transform.to_bare_matrix());
        self.draw_status_bar(data, ctx, &mut canvas, tps, sim_duration, fps, frame_duration, nodes, lines);
        canvas.finish(ctx).unwrap();
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
        if ctx.mouse.button_pressed(MouseButton::Left) {
            self.update_drag(data, pos, delta);
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.camera.position += Distance::new((width - self.width) / 2.0, (height - self.height) / 2.0);
        self.width = width;
        self.height = height;
    }

    pub fn update(&mut self, data: &mut Data, ctx: &mut Context) {
        let mouse_pos = self.offset_to_camera(Position::from(ctx.mouse.position()));
        self.update_under_mouse(data, mouse_pos);
    }
}
