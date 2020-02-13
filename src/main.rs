use hecs::World;
use ggez::{Context, ContextBuilder, GameResult, event::EventHandler, graphics::{Rect, Mesh, DrawMode, DrawParam, WHITE, BLACK}};


#[derive(Debug, Copy, Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

fn hello_world(world: &mut World) {
    for (_, position) in &mut world.query::<&Position>() {
        println!("Hello, {:?}", &position);
    }
}

fn draw(world: &mut World, ctx: &mut Context, lerp: f64) {
    for (_, (vel, pos)) in &mut world.query::<(Option<&Velocity>, &Position)>() {
        let x = vel.map(|vel| pos.x + vel.x * lerp as f32).unwrap_or(pos.x);
        let y = vel.map(|vel| pos.y + vel.y * lerp as f32).unwrap_or(pos.y);
        let square = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(x, y, 2.0, 2.0), BLACK).unwrap();
        ggez::graphics::draw(ctx, &square, DrawParam::default()).unwrap();
    }
}

fn update_pos(world: &mut World, delta: f64) {
    println!("Updating {:?}", delta);
    for (_, (vel, pos)) in &mut world.query::<(&Velocity, &mut Position)>() {
        pos.x += vel.x * delta as f32;
        pos.y += vel.y * delta as f32;
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
}

impl Ui {
    pub fn new(_ctx: &mut Context) -> Ui {
        let mut world = World::new();
     
        world.spawn((Position { x: 40.0, y: 70.0 },));
        world.spawn((Position { x: 20.0, y: 50.0 }, Velocity { x: 10.0, y: 20.0 }));
        // Load/create resources here: images, fonts, sounds, etc.
        Ui { world }
    }
}

impl EventHandler for Ui {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, 1) {
            update_pos(&mut self.world, 1.0);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        ggez::graphics::clear(ctx, WHITE);

        // Draw code here...
        let lerp = ggez::timer::duration_to_f64(ggez::timer::remaining_update_time(ctx));
        hello_world(&mut self.world);
        draw(&mut self.world, ctx, lerp);

        ggez::graphics::present(ctx)
    }
}
