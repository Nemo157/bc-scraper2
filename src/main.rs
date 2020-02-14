use hecs::World;
use std::time::{Instant, Duration};
use ggez::{Context, ContextBuilder, GameResult, event::EventHandler, graphics::{Rect, Mesh, DrawMode, DrawParam, WHITE, BLACK}};
use rand::distributions::{Distribution, Uniform};

use phys::{Position, Velocity, Acceleration};

mod phys;

const SIM_FREQ: u64 = 20;
const SIM_TIME: Duration = Duration::from_millis(1000 / SIM_FREQ);

fn draw(world: &mut World, ctx: &mut Context, delta: Duration) {
    for (_, (pos, vel_acc)) in &mut world.query::<(&Position, Option<(&Velocity, Option<&Acceleration>)>)>() {
        let pos = vel_acc.map(|(vel, acc)| pos + acc.map(|acc| vel + (acc / 2.0) * delta).unwrap_or(*vel) * delta).unwrap_or(*pos);
        let square = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(pos.0.x, pos.0.y, 2.0, 2.0), BLACK).unwrap();
        ggez::graphics::draw(ctx, &square, DrawParam::default()).unwrap();
    }
}

fn update_pos(world: &mut World, delta: Duration) {
    for (_, (mut pos, vel)) in &mut world.query::<(&mut Position, &Velocity)>() {
        pos += vel * delta;
    }
}

fn update_vel(world: &mut World, delta: Duration) {
    for (_, (mut vel, accel)) in &mut world.query::<(&mut Velocity, &Acceleration)>() {
        vel += accel * delta;
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

impl Ui {
    pub fn new(_ctx: &mut Context) -> Ui {
        let mut world = World::new();

        let mut rng = rand::thread_rng();
        let positions = Uniform::new(Position::new(200.0, 200.0), Position::new(400.0, 400.0));
        let velocities = Uniform::new(Velocity::new(-10.0, -10.0), Velocity::new(10.0, 10.0));
        let accelerations = Uniform::new(Acceleration::new(-10.0, -10.0), Acceleration::new(10.0, 10.0));

        let mut entities = Vec::new();
        for _ in 0..10 {
            for _ in 0..10 {
                entities.push(world.spawn((
                    positions.sample(&mut rng),
                    velocities.sample(&mut rng),
                    accelerations.sample(&mut rng),
                )));
            }
        }

        // Load/create resources here: images, fonts, sounds, etc.
        Ui { world, last_update: Instant::now() }
    }
}

impl EventHandler for Ui {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, SIM_FREQ as u32) {
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
