use url::Url;
use eyre::Error;

pub(crate) struct Scraper {
    client: crate::web::Client,
}

impl Scraper {
    pub(crate) fn new(client: crate::web::Client) -> Self {
        Self { client }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    pub(crate) fn scrape_album(&self, url: &Url) {
        let data = self.client.get(url)?;
        tracing::info!("data length: {}", data.len());
    }
}
