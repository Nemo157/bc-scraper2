use hecs::World;
use std::time::{Instant, Duration};
use ggez::{Context, ContextBuilder, GameResult, event::EventHandler, input::mouse::MouseButton, graphics::{Rect, Mesh, DrawMode, DrawParam, WHITE, BLACK, Color}};
use rand::{seq::SliceRandom, distributions::{Distribution, Uniform}};
use rand_distr::Poisson;

use phys::{Position, Velocity, Acceleration};

mod phys;

const SIM_FREQ: u64 = 20;
const SIM_TIME: Duration = Duration::from_millis(1000 / SIM_FREQ);
const LIGHT_RED: Color = Color::new(1.0, 0.0, 0.0, 0.2);

fn draw(world: &mut World, ctx: &mut Context, delta: Duration) {
    for (_, (square, pos, vel_acc)) in &mut world.query::<(&Mesh, &Position, Option<hecs::Without<UnderMouse, (&Velocity, Option<&Acceleration>)>>)>() {
        let pos = vel_acc.map(|(vel, acc)| pos + acc.map(|acc| vel + (acc / 2.0) * delta).unwrap_or(*vel) * delta).unwrap_or(*pos);
        ggez::graphics::draw(ctx, square, DrawParam::from(([pos.0.x, pos.0.y],))).unwrap();
    }

    // for (_, rel) in &mut world.query::<&Relationship>() {
    //     let (pos1, pos2) = (
    //         world.get::<Position>(rel.from).unwrap(),
    //         world.get::<Position>(rel.to).unwrap(),
    //     );
    //     let line = Mesh::new_line(ctx, &[[pos1.0.x, pos1.0.y], [pos2.0.x, pos2.0.y]], 0.5, LIGHT_RED).unwrap();
    //     ggez::graphics::draw(ctx, &line, DrawParam::default()).unwrap();
    // }
}

fn update_pos(world: &mut World, delta: Duration) {
    for (_, (mut pos, vel)) in &mut world.query::<hecs::Without<UnderMouse, (&mut Position, &Velocity)>>() {
        pos += vel * delta;
    }
}

fn update_vel(world: &mut World, delta: Duration) {
    for (_, vel) in &mut world.query::<hecs::With<UnderMouse, &mut Velocity>>() {
        *vel = Velocity::default();
    }

    for (_, (mut vel, acc)) in &mut world.query::<hecs::Without<UnderMouse, (&mut Velocity, &Acceleration)>>() {
        vel *= 0.7;
        vel += acc * delta;
    }
}

fn repel(world: &mut World) {
    for (_, (pos1, mut acc1)) in &mut world.query::<(&Position, &mut Acceleration)>() {
        for (_, pos2) in &mut world.query::<&Position>() {
            let dist = pos1 - pos2;
            let dsq = (dist.0.x * dist.0.x + dist.0.y * dist.0.y).max(0.001);
            acc1 += Acceleration::from(dist.0 * 1000.0) / dsq;
        }
    }
}

fn attract(world: &mut World) {
    for (_, rel) in &mut world.query::<&Relationship>() {
        let (pos1, pos2) = (
            world.get::<Position>(rel.from).unwrap(),
            world.get::<Position>(rel.to).unwrap(),
        );
        // TODO: Unit for attraction
        let attraction = Acceleration::from((*pos2 - *pos1).0 * 2.0);
        *world.get_mut::<Acceleration>(rel.from).unwrap() += attraction;
        *world.get_mut::<Acceleration>(rel.to).unwrap() += -attraction;
    }
}

fn update_acc(world: &mut World) {
    for (_, acc) in &mut world.query::<&mut Acceleration>() {
        *acc = Acceleration::default();
    }
    repel(world);
    attract(world);
}

struct UnderMouse;

fn update_drag(world: &mut World, mouse_pos: [f32; 2]) {
    for (_, pos) in &mut world.query::<hecs::With<UnderMouse, &mut Position>>() {
        *pos = Position::from(mouse_pos);
    }
}

fn update_under_mouse(world: &mut World, mouse_pos: [f32; 2]) {
    let mouse_pos = Position::from(mouse_pos);

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

fn main() {
    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) =
       ContextBuilder::new("game_name", "author_name")
           .build()
           .unwrap();

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut ui = Ui::new(&mut ctx);

    // Run!
    match ggez::event::run(&mut ctx, &mut event_loop, &mut ui) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

struct Ui {
    world: World,
    last_update: Instant,
}

#[derive(Debug)]
struct Relationship {
    from: hecs::Entity,
    to: hecs::Entity,
}

impl Ui {
    pub fn new(ctx: &mut Context) -> Ui {
        let mut world = World::new();

        let mut rng = rand::thread_rng();
        let positions = Uniform::new(Position::new(200.0, 200.0), Position::new(400.0, 400.0));
        let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));

        let mut entities = Vec::new();
        for _ in 0..100 {
            let square = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(-5.0, -5.0, 10.0, 10.0), BLACK).unwrap();
            entities.push(world.spawn((
                square,
                positions.sample(&mut rng),
                velocities.sample(&mut rng),
                Acceleration::default(),
            )));
        }

        for (i, from) in entities.iter().enumerate() {
            let count: u64 = Poisson::new((i / 20 + 1) as f64).unwrap().sample(&mut rng);
            let count = count.min(entities.len() as u64 / 2) as usize;
            for to in entities.choose_multiple(&mut rng, count) {
                if from == to { continue; }
                world.spawn((Relationship { from: *from, to: *to },));
            }
        }

        // Load/create resources here: images, fonts, sounds, etc.
        Ui { world, last_update: Instant::now() }
    }
}

impl EventHandler for Ui {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
            let mouse_pos = ggez::input::mouse::position(ctx);
            update_drag(&mut self.world, mouse_pos.into())
        }

        while ggez::timer::check_update_time(ctx, SIM_FREQ as u32) {
            let mouse_pos = ggez::input::mouse::position(ctx);
            update_under_mouse(&mut self.world, mouse_pos.into());
            update_acc(&mut self.world);
            update_vel(&mut self.world, SIM_TIME);
            update_pos(&mut self.world, SIM_TIME);
            self.last_update = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        ggez::graphics::clear(ctx, WHITE);

        // Draw code here...
        let delta = self.last_update.elapsed();
        draw(&mut self.world, ctx, delta);

        ggez::graphics::present(ctx)
    }
}
