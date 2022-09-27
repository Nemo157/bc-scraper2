use eyre::Error;
use ggez::{event::EventHandler, input::{mouse::MouseButton, keyboard::{KeyCode, KeyMods}}, Context, ContextBuilder, GameResult, GameError, conf::WindowMode};
use std::time::{Duration, Instant};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use crossbeam::channel::{Sender, Receiver, TryRecvError};
use clap::Parser;

use opt::{
    phys::{Distance, Position, Velocity},
    data::{Album, User, Data, EntityData},
    sim,
};

mod ui;
mod background;
mod fps;


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
        ContextBuilder::new("bc-scraper2", "mind your own bizness")
        .window_mode(WindowMode {
            resizable: true,
            ..WindowMode::default()
        })
        .build()?;

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
        ui.data.spawn_random(albums, users);
    }

    // Run!
    ggez::event::run(&mut ctx, &mut event_loop, &mut ui)?;
}

struct Ui {
    data: Data,
    last_update: Instant,
    last_mouse_position: Position,
    // target 5 seconds at 60 fps/20 tps
    fps: fps::Counter<300>,
    tps: fps::Counter<{5 * SIM_FREQ as usize}>,
    pause_sim: bool,
    // Order matters, sender and receiver must be dropped before background thread to tell it to shutdown
    to_scrape_tx: Sender<background::Request>,
    scraped_rx: Receiver<background::Response>,
    _background: background::Thread,
}

impl Ui {
    #[fehler::throws]
    pub fn new(ctx: &mut Context) -> Ui {
        let mut data = Data::default();

        ui::init(&mut data, ctx);

        let (scraped_tx, scraped_rx) = crossbeam::channel::bounded(1);
        let (to_scrape_tx, to_scrape_rx) = crossbeam::channel::unbounded();

        let _background = background::Thread::spawn(to_scrape_rx, scraped_tx)?;

        Ui {
            data,
            last_update: Instant::now(),
            fps: fps::Counter::default(),
            tps: fps::Counter::default(),
            last_mouse_position: Position::new(0.0, 0.0),
            pause_sim: false,
            to_scrape_tx,
            scraped_rx,
            _background,
        }
    }
}

impl EventHandler for Ui {
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Space => {
                self.pause_sim ^= true;
            }
            KeyCode::Escape => {
                ggez::event::quit(ctx);
            }
            KeyCode::Q if keymods.contains(KeyMods::CTRL) => {
                ggez::event::quit(ctx);
            }
            _ => {}
        }
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        ui::mouse_down(&mut self.data, ctx, button, Position::new(x, y));
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if let Some(entity) = ui::mouse_up(&mut self.data, ctx, button, Position::new(x, y)) {
            match &entity.data {
                EntityData::Album(Album { url, .. }) => {
                    self.to_scrape_tx.send(background::Request::Album { url: url.clone() }).unwrap();
                }
                EntityData::User(User { url, .. }) => {
                    self.to_scrape_tx.send(background::Request::User { url: url.clone() }).unwrap();
                }
            }
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        self.last_mouse_position = Position::new(x, y);
        ui::mouse_motion(
            &mut self.data,
            ctx,
            self.last_mouse_position,
            Distance::new(dx, dy),
        );
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        ui::mouse_wheel(
            &mut self.data,
            ctx,
            self.last_mouse_position,
            Velocity::new(x, y),
        );
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        ui::resize(&mut self.data, ctx, width, height);
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, SIM_FREQ as u32) {
            ui::update(&mut self.data, ctx);
            if !self.pause_sim {
                sim::update(&mut self.data, SIM_TIME);
                self.tps.tick();
            }
            match self.scraped_rx.try_recv() {
                Ok(response) => match response {
                    background::Response::Fans(album, users) => {
                        for user in users {
                            self.data.add_relationship(&album, &user);
                        }
                    }
                    background::Response::Collection(user, albums) => {
                        for album in albums {
                            self.data.add_relationship(&album, &user);
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

    #[fehler::throws(GameError)]
    fn draw(&mut self, ctx: &mut Context) {
        let delta = if self.pause_sim { Duration::default() } else { self.last_update.elapsed() };
        ui::draw(&mut self.data, ctx, delta, self.tps.value(), self.fps.value());
        ggez::graphics::present(ctx)?;
        self.fps.tick();
    }
}
