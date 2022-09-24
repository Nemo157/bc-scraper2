use eyre::Error;
use ggez::{event::EventHandler, input::mouse::MouseButton, Context, ContextBuilder, GameResult};
use hecs::World;
use std::time::{Duration, Instant};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use crossbeam::channel::{Sender, Receiver, TryRecvError};
use clap::Parser;

use phys::{Distance, Position, Velocity};
use data::{Album, User};

mod ui;
mod background;


const SIM_FREQ: u64 = 20;
const SIM_TIME: Duration = Duration::from_millis(1000 / SIM_FREQ);

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(long("user"), value_name("username"))]
    users: Vec<String>,
    #[arg(long("album"), value_name("url"))]
    albums: Vec<String>,
    #[arg(long, value_names(["albums", "users"]), num_args(2))]
    random: Vec<u64>,
}

#[fehler::throws]
fn main() {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(tracing_tree::HierarchicalLayer::new(2))
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .with_env_var("BC_SCRAPER_2_LOG")
                .from_env()?,
        )
        .with(tracing_error::ErrorLayer::default())
        .init();
    color_eyre::install()?;

    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) =
        ContextBuilder::new("bc-scraper2", "mind your own bizness").build()?;

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut ui = Ui::new(&mut ctx)?;

    for url in args.albums {
        ui.to_scrape_tx.send(background::Request::Album { url })?;
    }

    for username in args.users {
        ui.to_scrape_tx.send(background::Request::User { url: format!("https://bandcamp.com/{username}") })?;
    }

    if let [albums, users] = args.random[..] {
        ui.loader.spawn_random(&mut ui.world, albums, users);
    }

    // Run!
    ggez::event::run(&mut ctx, &mut event_loop, &mut ui)?;
}

struct Ui {
    world: World,
    last_update: Instant,
    last_mouse_position: Position,
    loader: data::Loader,
    // Order matters, sender and receiver must be dropped before background thread to tell it to shutdown
    to_scrape_tx: Sender<background::Request>,
    scraped_rx: Receiver<background::Response>,
    _background: background::Thread,
}

impl Ui {
    #[fehler::throws]
    pub fn new(ctx: &mut Context) -> Ui {
        let mut world = World::new();

        let loader = data::Loader::new();

        ui::init(&mut world, ctx);

        let (scraped_tx, scraped_rx) = crossbeam::channel::bounded(1);
        let (to_scrape_tx, to_scrape_rx) = crossbeam::channel::unbounded();

        let _background = background::Thread::spawn(to_scrape_rx, scraped_tx)?;

        Ui {
            world,
            last_update: Instant::now(),
            last_mouse_position: Position::new(0.0, 0.0),
            loader, 
            to_scrape_tx,
            scraped_rx,
            _background,
        }
    }
}

impl EventHandler for Ui {
    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        ui::mouse_down(&mut self.world, ctx, button, Position::new(x, y));
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if let Some(entity) = ui::mouse_up(&mut self.world, ctx, button, Position::new(x, y)) {
            if let Ok(album) = self.world.query_one_mut::<&Album>(entity) {
                self.to_scrape_tx.send(background::Request::Album { url: album.url.clone() }).unwrap();
            } else if let Ok(user) = self.world.query_one_mut::<&User>(entity) {
                self.to_scrape_tx.send(background::Request::User { url: user.url.clone() }).unwrap();
            }
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        self.last_mouse_position = Position::new(x, y);
        ui::mouse_motion(
            &mut self.world,
            ctx,
            self.last_mouse_position,
            Distance::new(dx, dy),
        );
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        ui::mouse_wheel(
            &mut self.world,
            ctx,
            self.last_mouse_position,
            Velocity::new(x, y),
        );
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, SIM_FREQ as u32) {
            ui::update(&mut self.world, ctx);
            sim::update(&mut self.world, SIM_TIME);
            match self.scraped_rx.try_recv() {
                Ok(response) => match response {
                    background::Response::Fans(album, users) => {
                        for user in users {
                            self.loader.add_relationship(&mut self.world, &album, &user);
                        }
                    }
                    background::Response::Collection(user, albums) => {
                        for album in albums {
                            self.loader.add_relationship(&mut self.world, &album, &user);
                        }
                    }
                    background::Response::Album(_) | background::Response::User(_) => {
                        // do nothing for now
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    panic!("background thread ded?");
                }
            }
            self.last_update = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = self.last_update.elapsed();
        ui::draw(&mut self.world, ctx, delta);
        ggez::graphics::present(ctx)
    }
}
