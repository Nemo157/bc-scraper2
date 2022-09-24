use crossbeam::channel::{Sender, Receiver};
use eyre::Error;
use url::Url;
use std::cell::RefCell;
use crate::data::{Album, User};

mod scrape;
mod web;

#[derive(Debug)]
pub enum Request {
    User { url: String },
    Album { url: String },
}

#[derive(Debug)]
pub enum Response {
    User(User),
    Album(Album),
    Fans(Album, Vec<User>),
    Collection(User, Vec<Album>),
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
    scraped: Sender<Response>,
}

impl Background {
    #[fehler::throws]
    fn new(
        to_scrape: Receiver<Request>,
        scraped: Sender<Response>,
    ) -> Self {
        let scraper = self::scrape::Scraper::new(self::web::Client::new()?);
        Self {
            scraper,
            to_scrape,
            scraped,
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
            Request::User { url } => {
                let user = RefCell::new(None);
                self.scraper.scrape_fan(&Url::parse(&url)?, |fan| {
                    self.scraped.send(Response::User(fan.clone())).unwrap();
                    user.replace(Some(fan));
                }, |collection| {
                    self.scraped.send(Response::Collection(user.borrow().clone().unwrap(), collection)).unwrap();
                })?;
            }
            Request::Album { url } => {
                let album = RefCell::new(None);
                self.scraper.scrape_album(&Url::parse(&url)?, |new_album| {
                    self.scraped.send(Response::Album(new_album.clone())).unwrap();
                    album.replace(Some(new_album));
                }, |fans| {
                    self.scraped.send(Response::Fans(album.borrow().clone().unwrap(), fans)).unwrap();
                })?;
            }
        }
    }
}
