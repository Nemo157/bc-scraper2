use url::Url;
use eyre::Error;

pub(crate) struct Scraper {
    client: crate::web::Client,
}

trait JsonExt {
    #[fehler::throws]
    fn parse_json<T: serde::de::DeserializeOwned>(&self) -> T;
}

impl JsonExt for str {
    #[fehler::throws]
    fn parse_json<T: serde::de::DeserializeOwned>(&self) -> T {
        serde_json::from_str(self)?
    }
}

trait DocumentExt {
    #[fehler::throws]
    fn try_select(&self, selector: &str) -> Vec<scraper::ElementRef<'_>>;

    #[fehler::throws]
    fn try_select_one(&self, selector: &str) -> scraper::ElementRef<'_>;
}

impl DocumentExt for scraper::Html {
    #[fehler::throws]
    #[tracing::instrument(skip(self))]
    fn try_select(&self, selector: &str) -> Vec<scraper::ElementRef<'_>> {
        let s = scraper::Selector::parse(selector).map_err(|e| eyre::eyre!("{e:?}"))?;
        self.select(&s).collect()
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self))]
    fn try_select_one(&self, selector: &str) -> scraper::ElementRef<'_> {
        let s = scraper::Selector::parse(selector).map_err(|e| eyre::eyre!("{e:?}"))?;
        self.select(&s).next().ok_or_else(|| eyre::eyre!("missing element for {selector}"))?
    }
}

#[derive(Debug)]
struct Page {
    properties: Properties,
    collectors: Collectors,
}

#[derive(Debug, serde::Deserialize)]
struct Properties {
    item_type: String,
    item_id: i64,
}

#[derive(Debug, serde::Deserialize)]
struct Collectors {
    band_thanks_text: String,
    more_reviews_available: bool,
    more_thumbs_available: bool,
    reviews: Vec<Review>,
    shown_reviews: Vec<Review>,
    thumbs: Vec<Fan>,
    shown_thumbs: Vec<Fan>,
}

#[derive(Debug, serde::Deserialize)]
struct Review {
    fan_id: i64,
    fav_track_title: Option<String>,
    image_id: i64,
    name: String,
    token: String,
    username: String,
    why: String,
}

#[derive(Debug, serde::Deserialize)]
struct Fan {
    fan_id: i64,
    image_id: i64,
    name: String,
    username: String,
    token: String,
}

#[derive(Debug, serde::Deserialize)]
struct Thumbs {
    results: Vec<Fan>,
    more_available: bool,
}

impl Scraper {
    pub(crate) fn new(client: crate::web::Client) -> Self {
        Self { client }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    pub(crate) fn scrape_album(&self, url: &Url) {
        let page = self.scrape_album_page(url)?;

        tracing::info!("thumbs: {}", page.collectors.thumbs.len());
        tracing::info!("shown thumbs: {}", page.collectors.shown_thumbs.len());
        tracing::info!("more thumbs: {}", page.collectors.more_thumbs_available);

        if page.collectors.more_thumbs_available {
            self.scrape_collectors_api(&url.join("/api/tralbumcollectors/2/thumbs")?, &page.properties, &page.collectors.thumbs.last().unwrap().token)?;
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    fn scrape_album_page(&self, url: &Url) -> Page {
        let data = self.client.get(url)?;
        let document = scraper::Html::parse_document(&data);
        let properties = document.try_select_one("meta[name=bc-page-properties]")?.value().attr("content").ok_or_else(|| eyre::eyre!("missing data-blob"))?.parse_json()?;
        let collectors = document.try_select_one("#collectors-data")?.value().attr("data-blob").ok_or_else(|| eyre::eyre!("missing data-blob"))?.parse_json()?;
        Page {
            properties,
            collectors,
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    fn scrape_collectors_api(&self, url: &Url, props: &Properties, token: &str) -> Thumbs {
        self.client.post(url, &serde_json::json!({
            "tralbum_type": props.item_type,
            "tralbum_id": props.item_id,
            "token": token,
            "count": 100,
        }))?.parse_json()?
    }
}
