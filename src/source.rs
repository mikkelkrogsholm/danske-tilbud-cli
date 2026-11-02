//! Source adapters map responses into source-near records; they do no price math.
use std::future::Future;

use crate::offer::{Membership, PieceCount, PriceKind, Quantity, QuantityBasis, Store};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMetadata {
    pub id: String,
    pub name: String,
    pub chains: Vec<String>,
    /// Human-readable limits of this source’s coverage.
    pub coverage_limitation: String,
    pub capabilities: SourceCapabilities,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceCapabilities {
    pub store_filter: bool,
    pub date_filter: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FetchOptions {
    pub store_id: Option<String>,
    pub date: Option<chrono::NaiveDate>,
}

/// Adapters own transport, pagination and response mapping. A batch may contain
/// both records and errors when a source only partially succeeds.
pub trait SourceAdapter {
    fn metadata(&self) -> &SourceMetadata;
    fn fetch(&self, options: &FetchOptions) -> impl Future<Output = SourceBatch>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceBatch {
    /// True only when every selected scope has been fetched successfully.
    pub complete: bool,
    pub records: Vec<RawOffer>,
    pub coverage: Option<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum RawBoundary {
    Instant(String),
    Date(String),
    Unknown,
}

/// Decimal prices are strings in the source currency, never binary floats.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RawOffer {
    pub schema_version: u32,
    pub source_id: String,
    pub chain_id: String,
    pub store: Option<Store>,
    pub heading: Option<String>,
    pub description: Option<String>,
    pub currency: String,
    pub amount: Option<String>,
    pub before: Option<String>,
    pub quantity: Option<Quantity>,
    pub quantity_basis: Option<QuantityBasis>,
    pub pieces: Option<PieceCount>,
    pub starts: RawBoundary,
    pub ends: RawBoundary,
    pub membership: Membership,
    pub fetched_at: String,
    pub source_reference: String,
    pub ean: Option<String>,
    pub product_id: Option<String>,
    pub price_kind: PriceKind,
}
