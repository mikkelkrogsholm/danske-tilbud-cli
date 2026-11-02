//! REMA's publicly readable product feed; price selection stays source-specific.
use crate::{
    offer::{Membership, PriceKind, Quantity, QuantityBasis},
    source::{
        FetchOptions, RawBoundary, RawOffer, SourceAdapter, SourceBatch, SourceCapabilities,
        SourceMetadata,
    },
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use std::{collections::HashSet, time::Duration};

const LIMIT: usize = 100;
const MAX_PAGES: usize = 200;
pub struct RemaAdapter {
    client: reqwest::Client,
    base: String,
    metadata: SourceMetadata,
}
impl Default for RemaAdapter {
    fn default() -> Self {
        Self::new()
    }
}
impl RemaAdapter {
    pub fn new() -> Self {
        Self::with_base("https://api.digital.rema1000.dk", Duration::from_secs(10))
    }
    /// Fixture endpoint and timeout for deterministic transport tests.
    pub fn with_base(base: &str, timeout: Duration) -> Self {
        Self { client: reqwest::Client::builder().user_agent("danske-tilbud-cli/0.1").timeout(timeout).build().expect("HTTP client"), base: base.trim_end_matches('/').into(), metadata: SourceMetadata {
            id: "rema".into(), name: "REMA 1000".into(), chains: vec!["rema1000".into()],
            coverage_limitation: "Offentligt produktkatalog, ikke butiksspecifik tilgængelighed. Kun priser med kendt aktuelt tidsinterval bruges; kilden angiver ikke om intervalgrænser er inklusive. EAN er kun med, når entydigt oplyst.".into(),
            capabilities: SourceCapabilities::default(),
        }}
    }
}
impl SourceAdapter for RemaAdapter {
    fn metadata(&self) -> &SourceMetadata {
        &self.metadata
    }
    async fn fetch(&self, options: &FetchOptions) -> SourceBatch {
        let mut batch = SourceBatch::default();
        if options.store_id.is_some() || options.date.is_some() {
            batch
                .errors
                .push("REMA understøtter ikke butiks- eller datofilter".into());
            return batch;
        }
        let now = Utc::now();
        let mut seen = HashSet::new();
        let mut seen_ids = HashSet::new();
        let mut expected_last = None;
        let mut expected_total = None;
        let mut count = 0usize;
        for page in 1..=MAX_PAGES {
            let response = match self
                .client
                .get(format!("{}/api/v3/products", self.base))
                .query(&[("per_page", LIMIT), ("page", page)])
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    batch
                        .errors
                        .push(format!("page {page}: HTTP {}", r.status()));
                    break;
                }
                Err(e) => {
                    batch
                        .errors
                        .push(format!("page {page}: HTTP request failed: {e}"));
                    break;
                }
            };
            let body: Value = match response.json().await {
                Ok(v) => v,
                Err(e) => {
                    batch
                        .errors
                        .push(format!("page {page}: invalid response: {e}"));
                    break;
                }
            };
            let parsed = (|| -> Result<(&Vec<Value>, usize, usize), String> {
                let rows = body["data"].as_array().ok_or("missing data array")?;
                let p = &body["meta"]["pagination"];
                let get = |key: &str| {
                    p[key]
                        .as_u64()
                        .and_then(|x| usize::try_from(x).ok())
                        .ok_or_else(|| format!("invalid pagination {key}"))
                };
                if get("current_page")? != page || get("per_page")? != LIMIT || rows.len() > LIMIT {
                    return Err("inconsistent page metadata".into());
                }
                let last = get("last_page")?;
                let total = get("total")?;
                if last == 0 || page > last || last != total.max(1).div_ceil(LIMIT) {
                    return Err("inconsistent pagination totals".into());
                }
                Ok((rows, last, total))
            })();
            let (rows, last, total) = match parsed {
                Ok(v) => v,
                Err(e) => {
                    batch.errors.push(format!("page {page}: {e}"));
                    break;
                }
            };
            if expected_last.is_some_and(|x| x != last)
                || expected_total.is_some_and(|x| x != total)
            {
                batch
                    .errors
                    .push(format!("page {page}: pagination changed during fetch"));
                break;
            }
            expected_last = Some(last);
            expected_total = Some(total);
            let signature = serde_json::to_string(rows).expect("JSON serializes");
            if !rows.is_empty() && !seen.insert(signature) {
                batch.errors.push(format!("page {page}: repeated page"));
                break;
            }
            count += rows.len();
            for row in rows {
                let product_id = row["id"].to_string();
                if !seen_ids.insert(product_id.clone()) {
                    batch
                        .errors
                        .push(format!("page {page}: repeated product {product_id}"));
                    continue;
                }
                match map_product(row, now) {
                    Ok(r) => batch.records.push(r),
                    Err(e) => batch
                        .errors
                        .push(format!("page {page} product {}: {e}", row["id"])),
                }
            }
            if page == last {
                if count != total {
                    batch
                        .errors
                        .push(format!("expected {total} products, received {count}"));
                }
                break;
            }
            if rows.len() != LIMIT {
                batch.errors.push(format!("page {page}: short page"));
                break;
            }
            if page == MAX_PAGES {
                batch.errors.push("page limit reached".into());
            }
        }
        batch.complete = batch.errors.is_empty() && expected_last.is_some();
        batch.coverage = Some(format!(
            "REMA public product feed: {count} products; {} mapped",
            batch.records.len()
        ));
        batch
    }
}
fn text(v: &Value) -> Option<String> {
    v.as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
}
fn number(v: &Value) -> Option<String> {
    match v {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) if Decimal::from_str_exact(s).is_ok() => Some(s.clone()),
        _ => None,
    }
}
fn quantity(underline: &str) -> Option<Quantity> {
    // Only a leading single number and unit followed by a separator/end qualifies.
    let mut words = underline.split_whitespace();
    let raw = words.next()?;
    let amount = Decimal::from_str_exact(&raw.replace(',', ".")).ok()?;
    if amount <= Decimal::ZERO {
        return None;
    }
    let unit = words.next()?;
    match unit.to_ascii_lowercase().trim_end_matches('.') {
        "gr" | "g" | "kg" | "ml" | "cl" | "l" | "ltr" | "stk" => (),
        _ => return None,
    }
    if words.next().is_some_and(|word| word != "/") {
        return None;
    }
    Some(Quantity::Exact {
        amount,
        unit: unit.into(),
    })
}
fn map_product(v: &Value, now: DateTime<Utc>) -> Result<RawOffer, String> {
    let id = match &v["id"] {
        Value::Number(n) => n.to_string(),
        Value::String(s) if !s.is_empty() => s.clone(),
        _ => return Err("missing product id".into()),
    };
    let mut description = Vec::new();
    if let Some(s) = text(&v["underline"]) {
        description.push(s);
    }
    if let Some(s) = text(&v["description"]) {
        description.push(s);
    }
    let q = v["underline"].as_str().and_then(quantity);
    let mut active = Vec::new();
    if let Some(prices) = v["prices"].as_array() {
        for p in prices {
            if !p.is_object() {
                return Err("invalid price entry".into());
            }
            if !p["price"].is_null() && number(&p["price"]).is_none() {
                return Err("invalid price amount".into());
            }
            let parse_boundary =
                |field: &str| -> Result<Option<DateTime<chrono::FixedOffset>>, String> {
                    if p[field].is_null() {
                        return Ok(None);
                    }
                    let raw = p[field]
                        .as_str()
                        .ok_or_else(|| format!("invalid {field}"))?;
                    DateTime::parse_from_rfc3339(raw)
                        .map(Some)
                        .map_err(|_| format!("invalid {field}"))
                };
            let start = parse_boundary("starting_at")?;
            let end = parse_boundary("ending_at")?;
            if let (Some(start), Some(end)) = (start, end) {
                if start > end {
                    return Err("price starts after it ends".into());
                }
                if start < now && now < end {
                    let promo = p["is_campaign"].as_bool() == Some(true)
                        || p["is_advertised"].as_bool() == Some(true);
                    active.push((promo, start, p));
                }
            }
        }
    } else if !v["prices"].is_null() {
        return Err("invalid prices".into());
    }
    active.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
    let selected = active.first();
    let conflict = selected.is_some_and(|first| {
        active
            .iter()
            .skip(1)
            .take_while(|x| x.0 == first.0 && x.1 == first.1)
            .any(|x| {
                let decimal =
                    |p: &Value| number(&p["price"]).and_then(|s| Decimal::from_str_exact(&s).ok());
                decimal(x.2) != decimal(first.2)
            })
    });
    if conflict {
        description.push(
            "Flere lige prioriterede aktuelle priser er modstridende; prisen er ukendt.".into(),
        );
    }
    let p = selected.map(|x| x.2);
    if let Some(p) = p {
        for (field, label) in [
            ("max_quantity", "Maksimumsantal"),
            ("price_over_max_quantity", "Pris over maksimum"),
            ("deposit", "Pant"),
        ] {
            if !p[field].is_null() {
                description.push(format!("{label}: {}", p[field]));
            }
        }
    }
    let boundary = |field: &str| {
        p.and_then(|v| text(&v[field]))
            .map(RawBoundary::Instant)
            .unwrap_or(RawBoundary::Unknown)
    };
    let ean = v["barcodes"]
        .as_array()
        .and_then(|a| if a.len() == 1 { text(&a[0]) } else { None });
    Ok(RawOffer {
        schema_version: 1,
        source_id: id.clone(),
        chain_id: "rema1000".into(),
        store: None,
        heading: text(&v["name"]),
        description: (!description.is_empty()).then(|| description.join("\n")),
        currency: "DKK".into(),
        price_unit: crate::source::PriceUnit::Major,
        amount: if conflict {
            None
        } else {
            p.and_then(|v| number(&v["price"]))
        },
        before: None,
        quantity_basis: q.as_ref().map(|_| QuantityBasis::Total),
        quantity: q,
        pieces: None,
        starts: boundary("starting_at"),
        ends: boundary("ending_at"),
        membership: Membership::Unknown,
        fetched_at: now.to_rfc3339(),
        source_reference: format!("https://api.digital.rema1000.dk/api/v3/products/{id}"),
        ean,
        product_id: Some(id),
        price_kind: if selected.is_some_and(|x| x.0) {
            PriceKind::Offer
        } else {
            PriceKind::Catalog
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn exact_boundary_and_unparseable_price_are_not_invented() {
        let now = DateTime::parse_from_rfc3339("2026-11-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut product = json!({"id":1,"prices":[{"price":20,"is_campaign":true,"starting_at":"2026-11-02T00:00:00Z","ending_at":"2026-11-07T00:00:00Z"}]});
        let mapped = map_product(&product, now).unwrap();
        assert!(mapped.amount.is_none());
        assert_eq!(mapped.price_kind, PriceKind::Catalog);
        product["prices"][0]["starting_at"] = json!("bad");
        assert!(map_product(&product, now).is_err());
        product["prices"][0]["starting_at"] = json!(null);
        assert!(map_product(&product, now).unwrap().amount.is_none());
        product["prices"][0]["price"] = json!("not a price");
        assert!(map_product(&product, now).is_err());
    }
    #[test]
    fn newest_campaign_wins_and_equal_decimal_values_are_not_conflicts() {
        let now = DateTime::parse_from_rfc3339("2026-11-03T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let p = json!({"id":2,"prices":[
            {"price":40,"is_campaign":false,"starting_at":"2026-11-02T00:00:00Z","ending_at":"2026-11-07T00:00:00Z"},
            {"price":10,"is_campaign":true,"starting_at":"2026-11-01T00:00:00Z","ending_at":"2026-11-07T00:00:00Z"},
            {"price":20,"is_campaign":true,"starting_at":"2026-11-02T00:00:00Z","ending_at":"2026-11-07T00:00:00Z"},
            {"price":"20.00","is_advertised":true,"starting_at":"2026-11-02T00:00:00Z","ending_at":"2026-11-07T00:00:00Z"}
        ]});
        let mapped = map_product(&p, now).unwrap();
        assert_eq!(mapped.price_kind, PriceKind::Offer);
        assert_eq!(mapped.amount.as_deref(), Some("20"));
    }
}
