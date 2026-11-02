//! Version 1 of the JSON offer contract. See `docs/offer-v1.md`.
use chrono::{DateTime, FixedOffset, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A single source record. Version identifies the JSON contract, not source data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub schema_version: u32,
    pub source: String,
    pub source_id: String,
    pub chain_id: String,
    pub store: Option<Store>,
    pub heading: String,
    pub description: Option<String>,
    pub price: Price,
    pub quantity: Option<Quantity>,
    pub quantity_basis: Option<QuantityBasis>,
    pub pieces: Option<PieceCount>,
    pub validity: Validity,
    pub membership: Membership,
    pub fetched_at: DateTime<FixedOffset>,
    pub source_reference: String,
    pub ean: Option<String>,
    pub product_id: Option<String>,
    pub price_kind: PriceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Store {
    pub store_id: Option<String>,
    pub name: Option<String>,
    pub location: Option<String>,
}

/// Prices are DKK in integer øre. `before_ore` is a source-stated comparison
/// price; its conditions or provenance may be unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    pub amount_ore: Option<u64>,
    pub before_ore: Option<u64>,
}

/// Decimal values are JSON strings to prevent binary floating-point rounding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Quantity {
    Exact {
        #[serde(with = "rust_decimal::serde::str")]
        amount: Decimal,
        unit: String,
    },
    Range {
        #[serde(with = "rust_decimal::serde::str")]
        min: Decimal,
        #[serde(with = "rust_decimal::serde::str")]
        max: Decimal,
        unit: String,
    },
}

/// Whether the measurement applies to one piece or the whole package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantityBasis {
    PerPiece,
    Total,
}

/// Count of pieces in a package, separate from the measured size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PieceCount {
    Exact { count: u64 },
    Range { min: u64, max: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validity {
    pub starts: TimeBoundary,
    pub ends: TimeBoundary,
}

/// A date is intentionally not converted to a time or assumed timezone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "precision", content = "value", rename_all = "snake_case")]
pub enum TimeBoundary {
    Instant(DateTime<FixedOffset>),
    Date(NaiveDate),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Membership {
    Unknown,
    Required,
    NotRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceKind {
    Offer,
    Catalog,
}
