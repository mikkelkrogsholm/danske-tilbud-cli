//! Generic normalization, validation and deterministic deduplication.
use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::{
    offer::{Offer, PieceCount, Price, Quantity, QuantityBasis, TimeBoundary, Validity},
    source::{FetchOptions, RawBoundary, RawOffer, SourceAdapter, SourceBatch, SourceMetadata},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitPrice {
    /// Decimal øre per kg, litre or piece; decimal division may round to Decimal precision.
    pub ore: Decimal,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedOffer {
    pub offer: Offer,
    pub unit_price: Option<UnitPrice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidRecord {
    pub record: RawOffer,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    pub source: SourceMetadata,
    pub coverage: Option<String>,
    pub source_errors: Vec<String>,
    pub invalid_records: Vec<InvalidRecord>,
    pub offers: Vec<NormalizedOffer>,
    pub duplicates: usize,
}

pub async fn run<A: SourceAdapter>(adapter: &A, options: &FetchOptions) -> RunResult {
    normalize_batch(adapter.metadata().clone(), adapter.fetch(options).await)
}

/// Equal identities keep the lexicographically first serialized normalized offer;
/// results and invalid records are stable under input reordering.
pub fn normalize_batch(source: SourceMetadata, batch: SourceBatch) -> RunResult {
    let mut valid = BTreeMap::<String, (String, NormalizedOffer)>::new();
    let mut invalid = Vec::new();
    let mut duplicates = 0;
    for record in batch.records {
        match normalize(&source.id, &record) {
            Ok(offer) => {
                let key = identity(&offer.offer);
                let value = serde_json::to_string(&offer.offer).expect("offer serializes");
                match valid.entry(key) {
                    std::collections::btree_map::Entry::Vacant(e) => {
                        e.insert((value, offer));
                    }
                    std::collections::btree_map::Entry::Occupied(mut e) => {
                        duplicates += 1;
                        if value < e.get().0 {
                            e.insert((value, offer));
                        }
                    }
                }
            }
            Err(reason) => invalid.push(InvalidRecord { record, reason }),
        }
    }
    invalid.sort_by_cached_key(|r| serde_json::to_string(&r.record).expect("record serializes"));
    RunResult {
        source,
        coverage: batch.coverage,
        source_errors: batch.errors,
        invalid_records: invalid,
        offers: valid.into_values().map(|(_, o)| o).collect(),
        duplicates,
    }
}

fn identity(offer: &Offer) -> String {
    // Serialize structured components to avoid delimiter collisions.
    serde_json::to_string(&(
        &offer.source,
        &offer.source_id,
        &offer.chain_id,
        &offer.store,
        &offer.validity,
    ))
    .expect("identity serializes")
}

fn money(value: &Option<String>) -> Result<Option<u64>, String> {
    value
        .as_ref()
        .map(|s| {
            let d = Decimal::from_str_exact(s).map_err(|_| format!("invalid price: {s}"))?;
            let ore = d.checked_mul(Decimal::from(100)).ok_or("price overflow")?;
            let rounded = ore.round();
            if ore.is_sign_negative() || (ore - rounded).abs() > Decimal::new(1, 8) {
                return Err("price must be nonnegative whole øre".into());
            }
            rounded.to_u64().ok_or_else(|| "price overflow".into())
        })
        .transpose()
}

fn boundary(raw: &RawBoundary) -> Result<TimeBoundary, String> {
    match raw {
        RawBoundary::Unknown => Ok(TimeBoundary::Unknown),
        RawBoundary::Date(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(TimeBoundary::Date)
            .map_err(|_| format!("invalid date: {s}")),
        RawBoundary::Instant(s) => DateTime::parse_from_rfc3339(s)
            .map(TimeBoundary::Instant)
            .map_err(|_| format!("invalid instant: {s}")),
    }
}

fn normalize(source: &str, r: &RawOffer) -> Result<NormalizedOffer, String> {
    if r.schema_version != 1 {
        return Err("unsupported schema version".into());
    }
    if [source, &r.source_id, &r.chain_id, &r.source_reference]
        .iter()
        .any(|s| s.trim().is_empty())
    {
        return Err("identity and source reference must be nonempty".into());
    }
    if r.currency != "DKK" {
        return Err(format!("unsupported currency: {}", r.currency));
    }
    let price = Price {
        amount_ore: money(&r.amount)?,
        before_ore: money(&r.before)?,
    };
    let starts = boundary(&r.starts)?;
    let ends = boundary(&r.ends)?;
    let reversed = match (&starts, &ends) {
        (TimeBoundary::Date(a), TimeBoundary::Date(b)) => a > b,
        (TimeBoundary::Instant(a), TimeBoundary::Instant(b)) => a > b,
        _ => false,
    };
    if reversed {
        return Err("validity starts after it ends".into());
    }
    let fetched_at = DateTime::parse_from_rfc3339(&r.fetched_at)
        .map_err(|_| "invalid fetched_at".to_string())?;
    if let Some(p) = &r.pieces {
        match p {
            PieceCount::Exact { count } if *count == 0 => {
                return Err("piece count must be positive".into());
            }
            PieceCount::Range { min, max } if *min == 0 || min > max => {
                return Err("invalid piece count range".into());
            }
            _ => (),
        }
    }
    let quantity = r.quantity.as_ref().map(normalize_quantity).transpose()?;
    if quantity.is_none() && r.quantity_basis.is_some() {
        return Err("quantity basis without quantity".into());
    }
    let unit_price = calculate_unit_price(
        price.amount_ore,
        quantity.as_ref(),
        r.quantity_basis.as_ref(),
        r.pieces.as_ref(),
    );
    let offer = Offer {
        schema_version: 1,
        source: source.into(),
        source_id: r.source_id.clone(),
        chain_id: r.chain_id.clone(),
        store: r.store.clone(),
        heading: r.heading.as_ref().filter(|s| !s.trim().is_empty()).cloned(),
        description: r.description.clone(),
        price,
        quantity,
        quantity_basis: r.quantity_basis.clone(),
        pieces: r.pieces.clone(),
        validity: Validity { starts, ends },
        membership: r.membership.clone(),
        fetched_at,
        source_reference: r.source_reference.clone(),
        ean: r.ean.clone(),
        product_id: r.product_id.clone(),
        price_kind: r.price_kind.clone(),
    };
    Ok(NormalizedOffer { offer, unit_price })
}

fn exact_product(value: Decimal, factor: Decimal) -> Option<Decimal> {
    let result = value.checked_mul(factor)?;
    if result.is_zero() || result.checked_div(factor)? != value {
        return None;
    }
    Some(result)
}

fn normalize_quantity(q: &Quantity) -> Result<Quantity, String> {
    let convert = |unit: &str| -> Result<(&'static str, Decimal), String> {
        match unit.trim().to_lowercase().as_str() {
            "kg" | "kilogram" => Ok(("kg", Decimal::ONE)),
            "g" | "gram" => Ok(("kg", Decimal::new(1, 3))),
            "l" | "liter" | "litre" => Ok(("l", Decimal::ONE)),
            "cl" => Ok(("l", Decimal::new(1, 2))),
            "ml" => Ok(("l", Decimal::new(1, 3))),
            "stk" | "stk." | "styk" | "stykker" | "pcs" => Ok(("stk", Decimal::ONE)),
            _ => Err(format!("unsupported unit: {unit}")),
        }
    };
    match q {
        Quantity::Exact { amount, unit } => {
            if *amount <= Decimal::ZERO {
                return Err("quantity must be positive".into());
            }
            let (unit, factor) = convert(unit)?;
            let amount = exact_product(*amount, factor)
                .ok_or("quantity conversion overflow or underflow")?;
            Ok(Quantity::Exact {
                amount,
                unit: unit.into(),
            })
        }
        Quantity::Range { min, max, unit } => {
            if *min <= Decimal::ZERO || min > max {
                return Err("invalid quantity range".into());
            }
            let (unit, factor) = convert(unit)?;
            let min =
                exact_product(*min, factor).ok_or("quantity conversion overflow or underflow")?;
            let max =
                exact_product(*max, factor).ok_or("quantity conversion overflow or underflow")?;
            Ok(Quantity::Range {
                min,
                max,
                unit: unit.into(),
            })
        }
    }
}

fn calculate_unit_price(
    price: Option<u64>,
    quantity: Option<&Quantity>,
    basis: Option<&QuantityBasis>,
    pieces: Option<&PieceCount>,
) -> Option<UnitPrice> {
    let Quantity::Exact { amount, unit } = quantity? else {
        return None;
    };
    let total = match basis? {
        QuantityBasis::Total => *amount,
        QuantityBasis::PerPiece => match pieces? {
            PieceCount::Exact { count } => exact_product(*amount, Decimal::from(*count))?,
            PieceCount::Range { .. } => return None,
        },
    };
    Some(UnitPrice {
        ore: Decimal::from(price?).checked_div(total)?,
        unit: unit.clone(),
    })
}
