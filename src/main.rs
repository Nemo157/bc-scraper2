use eyre::Error;
use ggez::{event::EventHandler, input::{mouse::MouseButton, keyboard::{KeyInput, KeyMods, KeyCode}}, Context, ContextBuilder, GameResult, GameError, conf::WindowMode};
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
use crate::ui::Ui;

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
    #[arg(long("artist"), value_name("url"))]
    artists: Vec<String>,
    #[arg(long, value_names(["albums", "users"]), num_args(2))]
    random: Vec<u64>,
}

fn main() -> eyre::Result<()> {
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
    let (mut ctx, event_loop) =
        ContextBuilder::new("bc-scraper2", "mind your own bizness")
        .window_mode(WindowMode {
            resizable: true,
            ..WindowMode::default()
        })
        .build()?;

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut ui = App::new(&mut ctx)?;

    for url in args.albums {
        ui.to_scrape_tx.send(background::Request::Album { url })?;
    }

    for username in args.users {
        ui.to_scrape_tx.send(background::Request::User { url: format!("https://bandcamp.com/{username}") })?;
    }

    for url in args.artists {
        ui.to_scrape_tx.send(background::Request::Artist { url })?;
    }

    if let [albums, users] = args.random[..] {
        ui.data.spawn_random(albums, users);
    }

    // Run!
    ggez::event::run(ctx, event_loop, ui)
}

struct App {
    ui: Ui,
    data: Data,
    last_update: Instant,
    last_mouse_position: Position,
    tps: fps::Counter<2>,
    fps: fps::Counter<120>,
    pause_sim: bool,
    // Order matters, sender and receiver must be dropped before background thread to tell it to shutdown
    to_scrape_tx: Sender<background::Request>,
    scraped_rx: Receiver<background::Response>,
    _background: background::Thread,
}

impl App {
    #[fehler::throws]
    pub fn new(ctx: &mut Context) -> Self {
        let (scraped_tx, scraped_rx) = crossbeam::channel::bounded(1);
        let (to_scrape_tx, to_scrape_rx) = crossbeam::channel::unbounded();

        let _background = background::Thread::spawn(to_scrape_rx, scraped_tx)?;

        Self {
            data: Data::default(),
            ui: Ui::new(ctx),
            last_update: Instant::now(),
            tps: fps::Counter::new(20.0),
            fps: fps::Counter::new(60.0),
            last_mouse_position: Position::new(0.0, 0.0),
            pause_sim: false,
            to_scrape_tx,
            scraped_rx,
            _background,
        }
    }
}

impl EventHandler for App {
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult<()> {
        match input.keycode {
            Some(KeyCode::Space) => {
                self.pause_sim ^= true;
            }
            Some(KeyCode::Escape) => {
                ctx.request_quit();
            }
            Some(KeyCode::Q) if input.mods.contains(KeyMods::CTRL) => {
                ctx.request_quit();
            }
            Some(KeyCode::L) => {
                self.ui.enable_lines ^= true;
            }
            Some(KeyCode::N) => {
                self.ui.enable_nodes ^= true;
            }
            _ => {}
        }
        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult<()> {
        self.ui.mouse_down(&mut self.data, ctx, button, Position::new(x, y));
        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult<()> {
        if let Some(entity) = self.ui.mouse_up(&mut self.data, ctx, button, Position::new(x, y)) {
            match &*entity.data {
                EntityData::Album(Album { url, .. }) => {
                    self.to_scrape_tx.send(background::Request::Album { url: url.clone() }).unwrap();
                }
                EntityData::User(User { url, .. }) => {
                    self.to_scrape_tx.send(background::Request::User { url: url.clone() }).unwrap();
                }
            }
        }
        Ok(())
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) -> GameResult<()> {
        self.last_mouse_position = Position::new(x, y);
        self.ui.mouse_motion(
            &mut self.data,
            ctx,
            self.last_mouse_position,
            Distance::new(dx, dy),
        );
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) -> GameResult<()> {
        self.ui.mouse_wheel(self.last_mouse_position, Velocity::new(x, y));
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult<()> {
        self.ui.resize(width, height);
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.ui.update(&mut self.data, ctx);
        if ctx.time.check_update_time(SIM_FREQ as u32) {
            if self.pause_sim {
                self.tps.reset_start();
            } else {
                self.tps.record(|| {
                    sim::update(&mut self.data, SIM_TIME);
                });
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
                    background::Response::Release(url) => {
                        self.to_scrape_tx.send(background::Request::Album { url }).unwrap();
                    }
                    background::Response::Album(Album { id, .. }) => {
                        if let Some(&id) = self.data.albums.get(&id) {
                            self.data.entities[id].is_scraped = true;
                        }
                    }
                    background::Response::User(User { id, .. }) => {
                        if let Some(&id) = self.data.users.get(&id) {
                            self.data.entities[id].is_scraped = true;
                        }
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
        let (fps, draw_duration) = (self.fps.per_second(), self.fps.inner_duration());
        self.fps.record(|| {
            self.ui.draw(&self.data, ctx, delta, self.tps.per_second(), self.tps.inner_duration(), fps, draw_duration);
        });
    }
}
