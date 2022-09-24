use url::Url;
use eyre::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Scraper {
    client: super::web::Client,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: u64,
    pub url: String,
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
struct AlbumPage {
    properties: Properties,
    collectors: Collectors,
}

#[derive(Debug, serde::Deserialize)]
struct Properties {
    item_type: String,
    item_id: u64,
}

#[derive(Debug, serde::Deserialize)]
struct Collectors {
    // TODO: load more reviews
    // more_reviews_available: bool,
    more_thumbs_available: bool,
    reviews: Vec<Review>,
    thumbs: Vec<Fan>,
}

#[derive(Debug, serde::Deserialize)]
struct Review {
    fan_id: u64,
    username: String,
}

#[derive(Debug, serde::Deserialize)]
struct Fan {
    fan_id: u64,
    username: String,
    token: String,
}

#[derive(Debug, serde::Deserialize)]
struct Thumbs {
    results: Vec<Fan>,
    more_available: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct CollectionItem {
    item_id: u64,
    item_url: String,
}

#[derive(Debug, serde::Deserialize)]
struct ItemCache {
    collection: HashMap<String, CollectionItem>,
}

#[derive(Debug, serde::Deserialize)]
struct CollectionData {
    last_token: String,
    sequence: Vec<String>
}

#[derive(Debug, serde::Deserialize)]
pub struct FanData {
    fan_id: u64,
}

#[derive(Debug, serde::Deserialize)]
struct FanPage {
    fan_data: FanData,
    collection_count: usize,
    collection_data: CollectionData,
    item_cache: ItemCache,
}

#[derive(Debug, serde::Deserialize)]
struct Collections {
    more_available: bool,
    last_token: String,
    items: Vec<CollectionItem>,
}

impl Scraper {
    pub(crate) fn new(client: super::web::Client) -> Self {
        Self { client }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self, on_album, on_fans), fields(%url))]
    pub(crate) fn scrape_album(&self, url: &Url, on_album: impl FnOnce(Album), mut on_fans: impl FnMut(Vec<User>)) {
        let page = self.scrape_album_page(url)?;

        let mut more_available = page.collectors.more_thumbs_available;
        on_album(Album {
            id: page.properties.item_id,
            url: url.to_string(),
        });

        let mut token = page.collectors.thumbs.last().unwrap().token.clone();
        on_fans(page.collectors.reviews.into_iter().map(|review| User { id: review.fan_id, username: review.username, }).collect());
        on_fans(page.collectors.thumbs.into_iter().map(|thumb| User { id: thumb.fan_id, username: thumb.username, }).collect());

        while more_available {
            let response = self.scrape_collectors_api(url, &page.properties, &token)?;
            token = response.results.last().unwrap().token.clone();
            more_available = response.more_available;
            on_fans(response.results.into_iter().map(|thumb| User { id: thumb.fan_id, username: thumb.username, }).collect());
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self, on_fan, on_collection))]
    pub(crate) fn scrape_fan(&self, username: &str, on_fan: impl FnOnce(User), mut on_collection: impl FnMut(Vec<Album>)) {
        let url = Url::parse("https://bandcamp.com")?.join(username)?;
        let mut page = self.scrape_fan_page(&url)?;

        on_fan(User { id: page.fan_data.fan_id, username: username.into() });

        let items = Result::<Vec<_>, _>::from_iter(page.collection_data.sequence.into_iter().map(|s| page.item_cache.collection.remove(&s).ok_or_else(|| eyre::eyre!("cache missing collection item"))))?;
        let mut last_token = page.collection_data.last_token;
        let mut more_available = items.len() < page.collection_count;
        on_collection(items.into_iter().map(|item| Album { id: item.item_id, url: item.item_url }).collect());

        while more_available {
            let response = self.scrape_collections_api(page.fan_data.fan_id, &last_token)?;
            more_available = response.more_available;
            last_token = response.last_token;
            on_collection(response.items.into_iter().map(|item| Album { id: item.item_id, url: item.item_url }).collect());
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    fn scrape_album_page(&self, url: &Url) -> AlbumPage {
        let data = self.client.get(url)?;
        let document = scraper::Html::parse_document(&data);
        let properties = document.try_select_one("meta[name=bc-page-properties]")?.value().attr("content").ok_or_else(|| eyre::eyre!("missing data-blob"))?.parse_json()?;
        let collectors = document.try_select_one("#collectors-data")?.value().attr("data-blob").ok_or_else(|| eyre::eyre!("missing data-blob"))?.parse_json()?;
        AlbumPage {
            properties,
            collectors,
        }
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%url))]
    fn scrape_fan_page(&self, url: &Url) -> FanPage {
        let data = self.client.get(url)?;
        let document = scraper::Html::parse_document(&data);
        document.try_select_one("#pagedata")?.value().attr("data-blob").ok_or_else(|| eyre::eyre!("missing data-blob"))?.parse_json()?
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self), fields(%base_url))]
    fn scrape_collectors_api(&self, base_url: &Url, props: &Properties, token: &str) -> Thumbs {
        let url = base_url.join("/api/tralbumcollectors/2/thumbs")?;
        self.client.post(&url, &serde_json::json!({
            "tralbum_type": props.item_type,
            "tralbum_id": props.item_id,
            "token": token,
            "count": 80,
        }))?.parse_json()?
    }

    #[fehler::throws]
    #[tracing::instrument(skip(self))]
    fn scrape_collections_api(&self, fan_id: u64, token: &str) -> Collections {
        let url = Url::parse("https://bandcamp.com/api/fancollection/1/collection_items")?;
        self.client.post(&url, &serde_json::json!({
            "fan_id": fan_id,
            "older_than_token": token,
            "count": 20,
        }))?.parse_json()?
    }
}
