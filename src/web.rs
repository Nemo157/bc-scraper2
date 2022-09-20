use anyhow::Error;
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
    pub(crate) fn get(&self, url: impl Into<Url>) -> String {
        let url = url.into();
        if let Some((retrieved, data)) = self
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
        {
            tracing::info!("got cached data for {url} from {retrieved}");
            data
        } else {
            tracing::info!("retrieving {url}");
            let data = self.client.get(url.clone()).send()?.text()?;
            self.cache.execute(
                "insert into pages values (:url, :retrieved, :data)",
                named_params!(":url": url, ":retrieved": Utc::now(), ":data": &data),
            )?;
            data
        }
    }
}
