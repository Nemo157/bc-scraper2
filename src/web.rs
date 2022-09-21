use eyre::Error;
use chrono::{offset::Utc, DateTime};
use rusqlite::{named_params, OptionalExtension};
use url::Url;

pub(crate) struct Client {
    client: reqwest::blocking::Client,
    cache: rusqlite::Connection,
}

impl Client {
    #[fehler::throws]
    pub(crate) fn new() -> Self {
        let cache = rusqlite::Connection::open("web-cache.sqlite")?;
        cache.execute(
            "
            create table if not exists pages (
                url text primary key,
                retrieved text not null,
                data text not null
            ) strict
        ",
            (),
        )?;

        Self {
            client: reqwest::blocking::Client::new(),
            cache,
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    pub(crate) fn get(&self, url: &Url) -> String {
        if let Some((retrieved, data)) = self.get_from_cache(url)? {
            tracing::info!("got cached data for {url} from {retrieved}");
            data
        } else {
            tracing::info!("retrieving {url}");
            let data = self.get_from_server(url)?;
            self.add_to_cache(url, &data)?;
            data
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    fn get_from_cache(&self, url: &Url) -> Option<(DateTime<Utc>, String)> {
        self
            .cache
            .query_row(
                "select retrieved, data from pages where url = :url",
                named_params!(":url": &url),
                |row| {
                    Ok((
                        row.get::<_, DateTime<Utc>>("retrieved")?,
                        row.get::<_, String>("data")?,
                    ))
                },
            )
            .optional()?
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    fn get_from_server(&self, url: &Url) -> String {
        self.client.get(url.clone()).send()?.text()?
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self, data), fields(%url, data_len=data.len()))]
    fn add_to_cache(&self, url: &Url, data: &str) {
        self.cache.execute(
            "insert into pages values (:url, :retrieved, :data)",
            named_params!(":url": url, ":retrieved": Utc::now(), ":data": &data),
        )?;
    }
}
