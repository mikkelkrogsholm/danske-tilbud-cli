//! Public BilkaToGo leaflet selection. No credentials are persisted or logged.
use crate::{
    offer::{Membership, PriceKind, Quantity, QuantityBasis, Store},
    source::{
        FetchOptions, PriceUnit, RawBoundary, RawOffer, SourceAdapter, SourceBatch,
        SourceCapabilities, SourceMetadata,
    },
};
use chrono::Utc;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::{collections::HashSet, time::Duration};

const PAGE_SIZE: usize = 1000;
const MAX_PAGES: usize = 100;
const ROOT_LIMIT: usize = 2_000_000;
const SCRIPT_LIMIT: usize = 6_000_000;
const RESPONSE_LIMIT: usize = 40_000_000;

pub struct BilkaToGoAdapter {
    client: reqwest::Client,
    site: String,
    query_base: Option<String>,
    metadata: SourceMetadata,
}
impl Default for BilkaToGoAdapter {
    fn default() -> Self {
        Self::new()
    }
}
impl BilkaToGoAdapter {
    pub fn new() -> Self {
        Self::with_endpoints("https://www.bilkatogo.dk", None, Duration::from_secs(30))
    }
    /// Separate fixture endpoints allow offline transport tests.
    pub fn with_endpoints(site: &str, query_base: Option<&str>, timeout: Duration) -> Self {
        Self { client: reqwest::Client::builder().timeout(timeout).user_agent("danske-tilbud-cli/0.1").build().expect("HTTP client"), site: site.trim_end_matches('/').into(), query_base: query_base.map(|s| s.trim_end_matches('/').into()), metadata: SourceMetadata {
            id: "bilkatogo".into(), name: "BilkaToGo".into(), chains: vec!["bilka".into()],
            coverage_limitation: "Aktuel tilbudsavis for valgt BilkaToGo lager/butik (standard 1653), kun varer i sortiment og på lager. Ikke hele sortimentet eller priser i alle fysiske Bilka-butikker; tilbudsperioder kan være ukendte.".into(),
            capabilities: SourceCapabilities { store_filter: true, date_filter: false },
        }}
    }
    async fn body(&self, mut response: reqwest::Response, cap: usize) -> Result<Vec<u8>, String> {
        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }
        if response.content_length().is_some_and(|n| n > cap as u64) {
            return Err("response exceeds size limit".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| "response read failed")? {
            if bytes.len() + chunk.len() > cap {
                return Err("response exceeds size limit".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
    async fn config(&self) -> Result<(String, String), String> {
        let response = self
            .client
            .get(format!("{}/", self.site))
            .send()
            .await
            .map_err(|_| "site request failed")?;
        let page = String::from_utf8(self.body(response, ROOT_LIMIT).await?)
            .map_err(|_| "invalid site encoding")?;
        let scripts = script_paths(&page);
        if scripts.len() > 32 {
            return Err("too many scripts".into());
        }
        let mut app = None;
        let mut key = None;
        for path in scripts.into_iter().rev() {
            let response = self
                .client
                .get(format!("{}{}", self.site, path))
                .send()
                .await
                .map_err(|_| "script request failed")?;
            let script = String::from_utf8(self.body(response, SCRIPT_LIMIT).await?)
                .map_err(|_| "invalid script encoding")?;
            if app.is_none() {
                app = config_value(&script, "NUXT_ENV_ALGOLIA_APP_ID", false);
            }
            if key.is_none() {
                key = config_value(&script, "NUXT_ENV_ALGOLIA_SEARCH_API_KEY", true);
            }
            if let (Some(a), Some(k)) = (&app, &key) {
                return Ok((a.clone(), k.clone()));
            }
        }
        Err("public search configuration missing".into())
    }
}
fn script_paths(page: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for tag in page.split('<').filter(|s| s.starts_with("script")) {
        let tag = tag.split('>').next().unwrap_or("");
        for quote in ['"', '\''] {
            if let Some(rest) = tag.split_once(&format!("src={quote}")).map(|x| x.1) {
                if let Some(path) = rest.split(quote).next() {
                    if path.starts_with("/_nuxt/")
                        && path.ends_with(".js")
                        && !path.contains("..")
                        && !paths.iter().any(|p| p == path)
                    {
                        paths.push(path.to_owned());
                    }
                }
            }
        }
    }
    paths
}
fn config_value(script: &str, name: &str, key: bool) -> Option<String> {
    let rest = script.split_once(name)?.1.strip_prefix(":\"")?;
    let val = rest.split('"').next()?;
    let valid = if key {
        val.len() == 32 && val.bytes().all(|b| b.is_ascii_hexdigit())
    } else {
        (6..=20).contains(&val.len())
            && val
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    };
    valid.then(|| val.to_owned())
}
impl SourceAdapter for BilkaToGoAdapter {
    fn metadata(&self) -> &SourceMetadata {
        &self.metadata
    }
    async fn fetch(&self, options: &FetchOptions) -> SourceBatch {
        let mut batch = SourceBatch::default();
        let store = options.store_id.as_deref().unwrap_or("1653");
        if options.date.is_some()
            || store.is_empty()
            || store.len() > 12
            || !store.bytes().all(|b| b.is_ascii_digit())
        {
            batch
                .errors
                .push("invalid store scope or unsupported date filter".into());
            return batch;
        }
        let mut config = match self.config().await {
            Ok(v) => v,
            Err(e) => {
                batch.errors.push(e);
                return batch;
            }
        };
        let mut refreshed = false;
        let mut ids = HashSet::new();
        let mut expected = None;
        let mut count = 0usize;
        let now = Utc::now().to_rfc3339();
        for page in 0..MAX_PAGES {
            let query = json!({"query":"", "filters":format!("isInAssortmentIn:{store} AND inStockStore:{store} AND nonsearchable:false AND isInCurrentLeaflet:true"), "hitsPerPage":PAGE_SIZE, "page":page, "attributesToRetrieve":["id","name","netcontent","storeData","isInCurrentLeaflet","ean","price","sales_price"]});
            let response = loop {
                let endpoint = self
                    .query_base
                    .clone()
                    .unwrap_or_else(|| format!("https://{}-dsn.algolia.net", config.0));
                let r = self
                    .client
                    .post(format!(
                        "{endpoint}/1/indexes/prod_BILKATOGO_PRODUCTS/query"
                    ))
                    .header("x-algolia-application-id", &config.0)
                    .header("x-algolia-api-key", &config.1)
                    .json(&query)
                    .send()
                    .await;
                match r {
                    Ok(r)
                        if (r.status().as_u16() == 401 || r.status().as_u16() == 403)
                            && !refreshed =>
                    {
                        refreshed = true;
                        match self.config().await {
                            Ok(c) => config = c,
                            Err(e) => {
                                batch.errors.push(e);
                                break None;
                            }
                        }
                    }
                    Ok(r) => break Some(r),
                    Err(_) => {
                        batch.errors.push(format!("page {page}: request failed"));
                        break None;
                    }
                }
            };
            let Some(response) = response else { break };
            let body = match self.body(response, RESPONSE_LIMIT).await {
                Ok(b) => b,
                Err(e) => {
                    batch.errors.push(format!("page {page}: {e}"));
                    break;
                }
            };
            let data: Value = match serde_json::from_slice(&body) {
                Ok(v) => v,
                Err(_) => {
                    batch.errors.push(format!("page {page}: invalid JSON"));
                    break;
                }
            };
            let parsed = (|| -> Result<(&Vec<Value>, usize, usize), &'static str> {
                let hits = data["hits"].as_array().ok_or("missing hits")?;
                let get = |k| {
                    data[k]
                        .as_u64()
                        .and_then(|v| usize::try_from(v).ok())
                        .ok_or("invalid pagination")
                };
                let total = get("nbHits")?;
                let pages = get("nbPages")?;
                if data["exhaustiveNbHits"].as_bool() != Some(true)
                    || get("page")? != page
                    || get("hitsPerPage")? != PAGE_SIZE
                    || pages != total.div_ceil(PAGE_SIZE)
                    || pages > MAX_PAGES
                    || hits.len() != (total.saturating_sub(page * PAGE_SIZE)).min(PAGE_SIZE)
                    || (total > 0 && page >= pages)
                {
                    return Err("inconsistent or incomplete pagination");
                }
                Ok((hits, total, pages))
            })();
            let (hits, total, pages) = match parsed {
                Ok(p) => p,
                Err(e) => {
                    batch.errors.push(format!("page {page}: {e}"));
                    break;
                }
            };
            if expected.is_some_and(|p| p != (total, pages)) {
                batch.errors.push(format!("page {page}: totals changed"));
                break;
            }
            expected = Some((total, pages));
            count += hits.len();
            for hit in hits {
                match map_product(hit, store, &now) {
                    Ok(record) if ids.insert(record.source_id.clone()) => {
                        batch.records.push(record)
                    }
                    Ok(_) => batch
                        .errors
                        .push(format!("page {page}: repeated product id")),
                    Err(e) => batch.errors.push(format!("page {page}: {e}")),
                }
            }
            if page + 1 >= pages {
                if count != total {
                    batch.errors.push("product count mismatch".into());
                }
                break;
            }
        }
        batch.complete = expected.is_some() && batch.errors.is_empty();
        batch.coverage = Some(format!(
            "BilkaToGo aktuel tilbudsavis, lager/butik {store}: {count} varer"
        ));
        batch
    }
}
fn text(v: &Value) -> Option<String> {
    v.as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
}
fn value_id(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.trim().is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn amount(v: &Value) -> Result<Option<String>, String> {
    match v {
        Value::Null => Ok(None),
        Value::Number(n) => Ok(Some(n.to_string())),
        Value::String(s) if Decimal::from_str_exact(s).is_ok() => Ok(Some(s.clone())),
        _ => Err("invalid selected-store price".into()),
    }
}
fn quantity(s: &str) -> Option<Quantity> {
    let s = s.trim();
    let i = s.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))?;
    let amount = Decimal::from_str_exact(&s[..i].replace(',', ".")).ok()?;
    let unit = s[i..].trim().to_ascii_lowercase();
    if amount <= Decimal::ZERO
        || !matches!(
            unit.as_str(),
            "g" | "gr" | "kg" | "ml" | "cl" | "l" | "ltr" | "stk"
        )
    {
        return None;
    }
    Some(Quantity::Exact { amount, unit })
}
fn map_product(v: &Value, store: &str, now: &str) -> Result<RawOffer, String> {
    let id = value_id(&v["id"])
        .or_else(|| value_id(&v["objectID"]))
        .ok_or("missing product id")?;
    if !v["storeData"].is_null() && !v["storeData"].is_object() {
        return Err("invalid store data".into());
    }
    let selected = &v["storeData"][store];
    if !selected.is_null() && !selected.is_object() {
        return Err("invalid store data".into());
    }
    let price = amount(&selected["price"])?;
    let before = amount(&selected["beforePrice"])?
        .filter(|s| Decimal::from_str_exact(s).is_ok_and(|d| !d.is_zero()));
    let leaflet = v["isInCurrentLeaflet"]
        .as_bool()
        .ok_or("missing leaflet flag")?;
    let mut desc = Vec::new();
    if let Some(s) = text(&v["netcontent"]) {
        desc.push(format!("Indhold: {s}"));
    }
    for (field, label) in [
        ("offerDescription", "Tilbudsvilkår"),
        ("offerMax", "Maksimumsantal"),
        ("offerMaxDescription", "Maksimumsvilkår"),
        ("offerCount", "Kildens antal"),
        ("multipromo", "Flerkøb"),
        ("multiPromoPrice", "Flerkøbspris, kildens værdi"),
    ] {
        let value = &selected[field];
        if !value.is_null() && value != "" && value != 0 {
            desc.push(format!(
                "{label}: {}",
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string())
            ));
        }
    }
    // Preserve source validity text without assuming timezone or inclusivity.
    for (field, label) in [
        ("offerFrom", "Kildens startdato"),
        ("offerUntil", "Kildens slutdato"),
    ] {
        if let Some(s) = text(&selected[field]) {
            desc.push(format!("{label}: {s}"));
        }
    }
    let q = v["netcontent"].as_str().and_then(quantity);
    Ok(RawOffer {
        schema_version: 1,
        source_id: id.clone(),
        chain_id: "bilka".into(),
        store: Some(Store {
            store_id: Some(store.into()),
            name: None,
            location: None,
        }),
        heading: text(&v["name"]),
        description: (!desc.is_empty()).then(|| desc.join("\n")),
        currency: "DKK".into(),
        price_unit: PriceUnit::Minor,
        amount: price,
        before,
        quantity_basis: q.as_ref().map(|_| QuantityBasis::Total),
        quantity: q,
        pieces: None,
        starts: RawBoundary::Unknown,
        ends: RawBoundary::Unknown,
        membership: Membership::Unknown,
        fetched_at: now.into(),
        source_reference: "https://www.bilkatogo.dk/".into(),
        ean: value_id(&v["ean"]),
        product_id: Some(id),
        price_kind: if leaflet {
            PriceKind::Offer
        } else {
            PriceKind::Catalog
        },
    })
}
