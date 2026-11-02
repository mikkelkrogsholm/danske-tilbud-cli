//! Tjek's Danish offer feed. Pagination and partial failures stay visible.
use crate::{
    offer::{Membership, PieceCount, PriceKind, Quantity, QuantityBasis, Store},
    source::{
        FetchOptions, RawBoundary, RawOffer, SourceAdapter, SourceBatch, SourceCapabilities,
        SourceMetadata,
    },
};
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use std::{collections::HashSet, time::Duration};

pub const CHAINS: &[(&str, &str)] = &[
    ("rema1000", "11deC"),
    ("netto", "9ba51"),
    ("lidl", "71c90"),
    ("foetex", "bdf5A"),
    ("bilka", "93f13"),
    ("365discount", "DWZE1w"),
    ("meny", "267e1m"),
    ("superbrugsen", "0b1e8"),
    ("kvickly", "c1edq"),
    ("brugsen", "d311fg"),
    ("spar", "88ddE"),
    ("loevbjerg", "65caN"),
    ("min-koebmand", "603dfL"),
    ("abc-lavpris", "70d42L"),
];
const LIMIT: usize = 100;
const MAX_PAGES: usize = 200;

pub struct TjekAdapter {
    client: reqwest::Client,
    base: String,
    chains: Vec<(&'static str, &'static str)>,
    metadata: SourceMetadata,
}
impl TjekAdapter {
    pub fn new() -> Self {
        Self::with_base("https://squid-api.tjek.com/v2", Duration::from_secs(10))
    }
    /// Configurable endpoint and timeout support offline fixtures.
    pub fn with_base(base: &str, timeout: Duration) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(
                    "danske-tilbud-cli/0.1 (+https://github.com/mikkelkrogsholm/danske-tilbud-cli)",
                )
                .timeout(timeout)
                .build()
                .expect("HTTP client"),
            base: base.trim_end_matches('/').into(),
            chains: CHAINS.to_vec(),
            metadata: SourceMetadata {
                id: "tjek".into(),
                name: "Tjek".into(),
                chains: CHAINS.iter().map(|(id, _)| (*id).to_owned()).collect(),
                coverage_limitation: "Tjek henter direkte tilbud for de fleste kæder. Bilka bruger katalog-fallback; selvstændige direkte tilbud ud over 1000 udelades. Kædedækning er ikke garanti for fuldstændige tilbud.".into(),
                capabilities: SourceCapabilities {
                    store_filter: true,
                    date_filter: true,
                },
            },
        }
    }
    /// Select canonical chain IDs; an unknown ID is an error, not an empty snapshot.
    pub fn select_chains(mut self, ids: &[&str]) -> Result<Self, String> {
        if ids.is_empty() {
            return Err("at least one chain is required".into());
        }
        let mut selected = Vec::new();
        for id in ids {
            let chain = CHAINS
                .iter()
                .find(|(name, _)| name == id)
                .ok_or_else(|| format!("unknown Tjek chain: {id}"))?;
            if !selected.contains(chain) {
                selected.push(*chain);
            }
        }
        self.chains = selected;
        Ok(self)
    }
    async fn page(
        &self,
        endpoint: &str,
        dealer: &str,
        catalog: Option<&str>,
        offset: usize,
    ) -> Result<Vec<Value>, String> {
        for attempt in 0..3 {
            let response = self
                .client
                .get(format!("{}/{}", self.base, endpoint))
                .query(&[
                    ("dealer_id", dealer.to_owned()),
                    ("limit", LIMIT.to_string()),
                    ("offset", offset.to_string()),
                ])
                .query(
                    &catalog
                        .map(|id| vec![("catalog_id", id)])
                        .unwrap_or_default(),
                )
                .send()
                .await;
            match response {
                Ok(r) if r.status().is_success() => {
                    return r
                        .json::<Vec<Value>>()
                        .await
                        .map_err(|e| format!("invalid response: {e}"));
                }
                Ok(r) if r.status().as_u16() == 429 && attempt < 2 => {
                    let raw = r
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|x| x.to_str().ok())
                        .ok_or("429 without Retry-After")?;
                    let delay = if let Ok(seconds) = raw.parse::<u64>() {
                        Duration::from_secs(seconds)
                    } else {
                        let at =
                            DateTime::parse_from_rfc2822(raw).map_err(|_| "invalid Retry-After")?;
                        (at.with_timezone(&Utc) - Utc::now())
                            .to_std()
                            .unwrap_or_default()
                    };
                    if delay > Duration::from_secs(5) {
                        return Err(format!("429 Retry-After too long: {raw}"));
                    }
                    tokio::time::sleep(delay).await;
                }
                Ok(r) if r.status().is_server_error() && attempt < 2 => {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt + 1))).await
                }
                Ok(r) => {
                    let status = r.status();
                    let body = r.text().await.unwrap_or_default();
                    let code = serde_json::from_str::<Value>(&body)
                        .ok()
                        .and_then(|v| v.get("name").and_then(Value::as_str).map(str::to_owned))
                        .unwrap_or_default();
                    return Err(format!("HTTP {status} {code}"));
                }
                Err(e) if attempt < 2 => {
                    let _ = e;
                    tokio::time::sleep(Duration::from_millis(100 * (attempt + 1))).await;
                }
                Err(e) => return Err(format!("HTTP request failed: {e}")),
            }
        }
        Err("retry limit reached".into())
    }
}
impl Default for TjekAdapter {
    fn default() -> Self {
        Self::new()
    }
}
impl TjekAdapter {
    async fn pages(
        &self,
        endpoint: &str,
        dealer: &str,
        catalog: Option<&str>,
    ) -> (Vec<Value>, Option<String>) {
        let mut all = Vec::new();
        let mut seen = HashSet::new();
        for page in 0..MAX_PAGES {
            let offset = page * LIMIT;
            let rows = match self.page(endpoint, dealer, catalog, offset).await {
                Ok(rows) => rows,
                Err(e) => return (all, Some(format!("offset {offset}: {e}"))),
            };
            if rows.len() > LIMIT {
                return (all, Some(format!("offset {offset}: page exceeds limit")));
            }
            let len = rows.len();
            let signature = serde_json::to_string(&rows).expect("response serializes");
            if !rows.is_empty() && !seen.insert(signature) {
                return (all, Some(format!("offset {offset}: repeated page")));
            }
            all.extend(rows);
            if len < LIMIT {
                return (all, None);
            }
        }
        (all, Some("page limit reached".into()))
    }
}
impl SourceAdapter for TjekAdapter {
    fn metadata(&self) -> &SourceMetadata {
        &self.metadata
    }
    async fn fetch(&self, options: &FetchOptions) -> SourceBatch {
        let mut batch = SourceBatch::default();
        let fetched_at = Utc::now().to_rfc3339();
        let mut statuses = Vec::new();
        for (chain, dealer) in &self.chains {
            let mut count = 0;
            let mut incomplete = false;
            let (direct, direct_error) = self.pages("offers", dealer, None).await;
            let capped = direct_error
                .as_ref()
                .is_some_and(|e| e.contains("PAGINATION_MAX_REACHED"));
            let mut groups = Vec::new();
            let scope = if capped {
                let (catalogs, error) = self.pages("catalogs", dealer, None).await;
                if let Some(e) = error {
                    batch.errors.push(format!("{chain} catalogs: {e}"));
                    incomplete = true;
                }
                if catalogs.is_empty() {
                    batch.errors.push(format!(
                        "{chain}: no catalogs returned; offer coverage unknown"
                    ));
                    incomplete = true;
                }
                for row in catalogs {
                    let catalog = (|| -> Result<(String, Option<String>), String> {
                        let id = string(&row, "id")?.to_owned();
                        if string(&row, "dealer_id")? != *dealer
                            || string(&row["dealer"], "id")? != *dealer
                            || string(&row["dealer"]["country"], "id")? != "DK"
                        {
                            return Err(format!("catalog {id}: wrong dealer or country"));
                        }
                        Ok((id, nullable_string(&row, "store_id")?))
                    })();
                    match catalog {
                        Ok((id, store)) => {
                            let (rows, error) = self.pages("offers", dealer, Some(&id)).await;
                            if let Some(e) = error {
                                batch.errors.push(format!("{chain} catalog {id}: {e}"));
                                incomplete = true;
                            }
                            groups.push((Some(id), store, rows));
                        }
                        Err(e) => {
                            batch.errors.push(format!("{chain} catalog: {e}"));
                            incomplete = true;
                        }
                    }
                }
                "published catalogs; standalone direct offers beyond 1000 excluded"
            } else {
                if let Some(ref e) = direct_error {
                    batch.errors.push(format!("{chain} offers: {e}"));
                    incomplete = true;
                }
                if direct.is_empty() && direct_error.is_none() {
                    batch.errors.push(format!(
                        "{chain}: empty direct feed; offer coverage unknown"
                    ));
                    incomplete = true;
                }
                groups.push((None, None, direct));
                "direct feed"
            };
            for (catalog, catalog_store, rows) in groups {
                for row in rows {
                    let mapped = (|| -> Result<Option<RawOffer>, String> {
                        let mut offer = map_offer(&row, chain, dealer, &fetched_at)?;
                        if let Some(catalog) = &catalog {
                            if string(&row, "catalog_id")? != catalog {
                                return Err("wrong catalog".into());
                            }
                            if let Some(id) = &catalog_store {
                                if offer
                                    .store
                                    .as_ref()
                                    .and_then(|s| s.store_id.as_ref())
                                    .is_some_and(|s| s != id)
                                {
                                    return Err("offer/catalog store mismatch".into());
                                }
                                if offer.store.is_none() {
                                    offer.store = Some(Store {
                                        store_id: Some(id.clone()),
                                        name: None,
                                        location: None,
                                    });
                                }
                            }
                        }
                        if options.store_id.as_ref().is_some_and(|id| {
                            offer
                                .store
                                .as_ref()
                                .and_then(|s| s.store_id.as_ref())
                                .is_some_and(|s| s != id)
                        }) {
                            return Ok(None);
                        }
                        if let Some(date) = options.date {
                            let start = parse_time(row["run_from"].as_str().unwrap_or(""))?;
                            let end = parse_time(row["run_till"].as_str().unwrap_or(""))?;
                            let day_start = chrono_tz::Europe::Copenhagen
                                .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
                                .single()
                                .ok_or("invalid filter date")?;
                            let day_end = date
                                .succ_opt()
                                .and_then(|d| d.and_hms_opt(0, 0, 0))
                                .and_then(|d| {
                                    chrono_tz::Europe::Copenhagen
                                        .from_local_datetime(&d)
                                        .single()
                                })
                                .ok_or("invalid filter date")?;
                            if end < day_start || start >= day_end {
                                return Ok(None);
                            }
                        }
                        Ok(Some(offer))
                    })();
                    match mapped {
                        Ok(Some(offer)) => {
                            batch.records.push(offer);
                            count += 1;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            batch.errors.push(format!(
                                "{chain} offer {}: {e}",
                                row.get("id").and_then(Value::as_str).unwrap_or("?")
                            ));
                            incomplete = true;
                        }
                    }
                }
            }
            statuses.push(format!(
                "{chain}: {} ({count} records; {scope})",
                if incomplete {
                    "partial"
                } else {
                    "complete for scope"
                }
            ));
        }
        batch.complete = !self.chains.is_empty() && batch.errors.is_empty();
        if let Some(store) = &options.store_id {
            statuses.push(format!(
                "store filter: {store} (offers with unknown store included)"
            ));
        }
        if let Some(date) = options.date {
            statuses.push(format!("date filter: {date} (Europe/Copenhagen)"));
        }
        batch.coverage = Some(statuses.join("; "));
        batch
    }
}

fn string<'a>(v: &'a Value, name: &str) -> Result<&'a str, String> {
    v.get(name)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing/invalid {name}"))
}
fn nullable_string(v: &Value, name: &str) -> Result<Option<String>, String> {
    match v.get(name) {
        Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        _ => Err(format!("missing/invalid {name}")),
    }
}
fn number(v: &Value, name: &str) -> Result<Option<String>, String> {
    match v.get(name) {
        Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        _ => Err(format!("missing/invalid {name}")),
    }
}
fn parse_time(s: &str) -> Result<DateTime<chrono::FixedOffset>, String> {
    DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z")
        .or_else(|_| DateTime::parse_from_rfc3339(s))
        .map_err(|_| format!("invalid timestamp: {s}"))
}
fn boundary(v: &Value, field: &str) -> Result<RawBoundary, String> {
    match nullable_string(v, field)? {
        None => Ok(RawBoundary::Unknown),
        Some(s) => Ok(RawBoundary::Instant(parse_time(&s)?.to_rfc3339())),
    }
}
fn map_offer(v: &Value, chain: &str, dealer: &str, fetched: &str) -> Result<RawOffer, String> {
    let id = string(v, "id")?;
    if string(v, "dealer_id")? != dealer
        || string(&v["dealer"], "id")? != dealer
        || string(&v["dealer"]["country"], "id")? != "DK"
    {
        return Err(format!("offer {id}: wrong dealer or country"));
    }
    let pricing = v.get("pricing").ok_or("missing pricing")?;
    let q = v.get("quantity").ok_or("missing quantity")?;
    let (quantity, basis, pieces) = if q.is_null() {
        (None, None, None)
    } else {
        let unit = string(&q["unit"], "symbol")?.to_owned();
        let size = &q["size"];
        let a = number(size, "from")?;
        let b = number(size, "to")?;
        let quantity = match (a, b) {
            (Some(a), Some(b)) => {
                let a = Decimal::from_str_exact(&a).map_err(|_| "invalid quantity")?;
                let b = Decimal::from_str_exact(&b).map_err(|_| "invalid quantity")?;
                Some(if a == b {
                    Quantity::Exact { amount: a, unit }
                } else {
                    Quantity::Range {
                        min: a,
                        max: b,
                        unit,
                    }
                })
            }
            (None, None) => None,
            _ => return Err("incomplete quantity range".into()),
        };
        let p = &q["pieces"];
        let a = number(p, "from")?;
        let b = number(p, "to")?;
        let pieces = match (a, b) {
            (Some(a), Some(b)) => {
                let a = a.parse::<u64>().map_err(|_| "invalid piece count")?;
                let b = b.parse::<u64>().map_err(|_| "invalid piece count")?;
                Some(if a == b {
                    PieceCount::Exact { count: a }
                } else {
                    PieceCount::Range { min: a, max: b }
                })
            }
            (None, None) => None,
            _ => return Err("incomplete piece range".into()),
        };
        let basis = if quantity.is_some() && pieces.is_some() {
            Some(QuantityBasis::PerPiece)
        } else {
            None
        };
        (quantity, basis, pieces)
    };
    let store = nullable_string(v, "store_id")?.map(|id| Store {
        store_id: Some(id),
        name: None,
        location: None,
    });
    let mut description = nullable_string(v, "description")?;
    if !q.is_null() {
        let p = &q["pieces"];
        for field in ["min", "max"] {
            if let Some(value) = p.get(field) {
                if !value.is_null() {
                    let n = value
                        .as_u64()
                        .ok_or_else(|| format!("invalid pieces.{field}"))?;
                    let text = description.get_or_insert_with(String::new);
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    let label = if field == "max" {
                        "maksimumsgrænse"
                    } else {
                        "minimumsgrænse"
                    };
                    text.push_str(&format!("Kildens {label}: {n} stk."));
                }
            }
        }
    }
    Ok(RawOffer {
        schema_version: 1,
        source_id: id.into(),
        chain_id: chain.into(),
        store,
        heading: match v.get("heading") {
            Some(Value::Null) => None,
            Some(Value::String(s)) if s.trim().is_empty() => None,
            Some(Value::String(s)) => Some(s.clone()),
            _ => return Err("missing/invalid heading".into()),
        },
        description,
        currency: string(pricing, "currency")?.into(),
        amount: number(pricing, "price")?,
        before: number(pricing, "pre_price")?,
        quantity,
        quantity_basis: basis,
        pieces,
        starts: boundary(v, "run_from")?,
        ends: boundary(v, "run_till")?,
        membership: Membership::Unknown,
        fetched_at: fetched.into(),
        source_reference: format!("https://squid-api.tjek.com/v2/offers/{id}"),
        ean: None,
        product_id: None,
        price_kind: PriceKind::Offer,
    })
}
