use ggez::{event::EventHandler, input::mouse::MouseButton, Context, ContextBuilder, GameResult};
use hecs::World;
use std::time::{Duration, Instant};

mod data;
mod phys;
mod sim;
mod ui;

use crate::phys::{Distance, Position};

const SIM_FREQ: u64 = 20;
const SIM_TIME: Duration = Duration::from_millis(1000 / SIM_FREQ);

fn main() {
    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) = ContextBuilder::new("game_name", "author_name")
        .build()
        .unwrap();

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut ui = Ui::new(&mut ctx);

    // Run!
    match ggez::event::run(&mut ctx, &mut event_loop, &mut ui) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

struct Ui {
    world: World,
    last_update: Instant,
}

impl Ui {
    pub fn new(ctx: &mut Context) -> Ui {
        let mut world = World::new();

        data::spawn_random(&mut world);
        ui::init(&mut world, ctx);

        // Load/create resources here: images, fonts, sounds, etc.
        Ui {
            world,
            last_update: Instant::now(),
        }
    }
}

impl EventHandler for Ui {
    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        ui::mouse_down(&mut self.world, ctx, button, Position::new(x, y));
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        ui::mouse_up(&mut self.world, ctx, button, Position::new(x, y));
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        ui::mouse_motion(
            &mut self.world,
            ctx,
            Position::new(x, y),
            Distance::new(dx, dy),
        );
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, SIM_FREQ as u32) {
            ui::update(&mut self.world, ctx);
            sim::update(&mut self.world, SIM_TIME);
            self.last_update = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Draw code here...
        let delta = self.last_update.elapsed();
        ui::draw(&mut self.world, ctx, delta);
        ggez::graphics::present(ctx)
    }
}
