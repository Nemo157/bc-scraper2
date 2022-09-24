use crossbeam::channel::{Sender, Receiver};
use eyre::Error;
use url::Url;

mod scrape;
mod web;

#[derive(Debug)]
pub enum Request {
    User { username: String },
    Album { url: String },
}

#[derive(Debug)]
pub enum Response {
}

#[derive(Debug)]
pub struct Thread {
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Thread {
    #[fehler::throws]
    pub fn spawn(
        to_scrape: Receiver<Request>,
        scraped: Sender<Response>,
    ) -> Self {
        let background = Background::new(to_scrape, scraped)?;
        let thread = Some(std::thread::spawn(move || background.run()));
        Thread { thread }
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        if let Err(e) = self.thread.take().unwrap().join() {
            std::panic::resume_unwind(e);
        }
    }
}

#[derive(Debug)]
struct Background {
    scraper: self::scrape::Scraper,
    to_scrape: Receiver<Request>,
    _scraped: Sender<Response>,
}

impl Background {
    #[fehler::throws]
    fn new(
        to_scrape: Receiver<Request>,
        _scraped: Sender<Response>,
    ) -> Self {
        let scraper = self::scrape::Scraper::new(self::web::Client::new()?);
        Self {
            scraper,
            to_scrape,
            _scraped,
        }
    }

    fn run(&self) {
        for request in &self.to_scrape {
            if let Err(error) = self.handle_request(request) {
                tracing::error!(?error, "failed handling scrape request");
            }
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self))]
    fn handle_request(&self, request: Request) {
        match request {
            Request::User { username } => {
                self.scraper.scrape_fan(&username, |fan| {
                    tracing::info!("scrapped {fan:?}");
                }, |collection| {
                    tracing::info!("collection count {}", collection.len());
                })?;
            }
            Request::Album { url } => {
                self.scraper.scrape_album(&Url::parse(&url)?, |album| {
                    tracing::info!("scrapped {album:?}");
                }, |fans| {
                    tracing::info!("fans count {}", fans.len());
                })?;
            }
        }
    }
}
