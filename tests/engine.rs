use rust_decimal::Decimal;
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};
use tilbud::{
    engine::{normalize_batch, run},
    offer::{Membership, PieceCount, PriceKind, Quantity, QuantityBasis, Store},
    source::{
        FetchOptions, RawBoundary, RawOffer, SourceAdapter, SourceBatch, SourceCapabilities,
        SourceMetadata,
    },
};

fn metadata() -> SourceMetadata {
    SourceMetadata {
        chains: vec![],
        id: "kilde".into(),
        name: "Kilde".into(),
        coverage_limitation: "Eksempeldækning".into(),
        capabilities: SourceCapabilities::default(),
    }
}
fn raw(id: &str) -> RawOffer {
    RawOffer {
        schema_version: 1,
        source_id: id.into(),
        chain_id: "kæde".into(),
        store: None,
        heading: Some("Æbler og øl".into()),
        description: None,
        currency: "DKK".into(),
        price_unit: tilbud::source::PriceUnit::Major,
        amount: Some("10.00".into()),
        before: None,
        quantity: Some(Quantity::Exact {
            amount: Decimal::from(500),
            unit: "g".into(),
        }),
        quantity_basis: Some(QuantityBasis::Total),
        pieces: None,
        starts: RawBoundary::Date("2026-11-02".into()),
        ends: RawBoundary::Date("2026-11-08".into()),
        membership: Membership::Unknown,
        fetched_at: "2026-11-02T12:00:00+01:00".into(),
        source_reference: "reference".into(),
        ean: None,
        product_id: None,
        price_kind: PriceKind::Offer,
    }
}
fn batch(records: Vec<RawOffer>) -> SourceBatch {
    SourceBatch {
        complete: true,
        records,
        coverage: Some("første side".into()),
        errors: vec!["side to fejlede".into()],
    }
}
struct Offline {
    meta: SourceMetadata,
    batch: SourceBatch,
}
impl SourceAdapter for Offline {
    fn metadata(&self) -> &SourceMetadata {
        &self.meta
    }
    async fn fetch(&self, _: &FetchOptions) -> SourceBatch {
        self.batch.clone()
    }
}
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("offline adapter unexpectedly pending"),
    }
}
#[test]
fn offline_adapter_normalizes_and_preserves_partial_failure() {
    let adapter = Offline {
        meta: metadata(),
        batch: batch(vec![raw("1")]),
    };
    let result = block_on(run(&adapter, &FetchOptions::default()));
    assert_eq!(result.coverage.as_deref(), Some("første side"));
    assert_eq!(result.source_errors, ["side to fejlede"]);
    let o = &result.offers[0];
    assert_eq!(o.offer.heading.as_deref(), Some("Æbler og øl"));
    assert_eq!(o.offer.price.amount_ore, Some(1000));
    assert_eq!(
        o.offer.quantity,
        Some(Quantity::Exact {
            amount: Decimal::new(5, 1),
            unit: "kg".into()
        })
    );
    assert_eq!(o.unit_price.as_ref().unwrap().ore, Decimal::from(2000));
}
#[test]
fn pack_ranges_missing_and_zero_price() {
    let mut pack = raw("pack");
    pack.quantity = Some(Quantity::Exact {
        amount: Decimal::from(330),
        unit: "ml".into(),
    });
    pack.quantity_basis = Some(QuantityBasis::PerPiece);
    pack.pieces = Some(PieceCount::Exact { count: 6 });
    pack.amount = Some("19.80".into());
    let mut range = pack.clone();
    range.source_id = "range".into();
    range.quantity = Some(Quantity::Range {
        min: Decimal::from(300),
        max: Decimal::from(330),
        unit: "ml".into(),
    });
    let mut unknown_count = pack.clone();
    unknown_count.source_id = "unknown".into();
    unknown_count.pieces = None;
    let mut missing = pack.clone();
    missing.source_id = "missing".into();
    missing.amount = None;
    let mut zero = pack.clone();
    zero.source_id = "zero".into();
    zero.amount = Some("0".into());
    let r = normalize_batch(
        metadata(),
        batch(vec![range, pack, unknown_count, missing, zero]),
    );
    let get = |id: &str| r.offers.iter().find(|o| o.offer.source_id == id).unwrap();
    assert_eq!(
        get("pack").unit_price.as_ref().unwrap().ore,
        Decimal::from(1000)
    );
    assert!(get("range").unit_price.is_none());
    assert!(get("unknown").unit_price.is_none());
    assert_eq!(get("missing").offer.price.amount_ore, None);
    assert!(get("missing").unit_price.is_none());
    assert_eq!(get("zero").offer.price.amount_ore, Some(0));
    assert_eq!(get("zero").unit_price.as_ref().unwrap().ore, Decimal::ZERO);
}
#[test]
fn invalid_records_are_visible_and_do_not_suppress_valid_records() {
    let mut invalid_date = raw("date");
    invalid_date.ends = RawBoundary::Date("2026-02-30".into());
    let mut reversed = raw("reversed");
    reversed.ends = RawBoundary::Date("2026-01-01".into());
    let mut currency = raw("currency");
    currency.currency = "EUR".into();
    let mut fractional = raw("fractional");
    fractional.amount = Some("1.001".into());
    let mut unit = raw("unit");
    unit.quantity = Some(Quantity::Exact {
        amount: Decimal::ONE,
        unit: "pose".into(),
    });
    let mut mixed = raw("mixed");
    mixed.starts = RawBoundary::Instant("2026-12-01T12:00:00Z".into());
    let result = normalize_batch(
        metadata(),
        batch(vec![
            raw("valid"),
            invalid_date,
            reversed,
            currency,
            fractional,
            unit,
            mixed,
        ]),
    );
    assert_eq!(result.invalid_records.len(), 5);
    assert_eq!(result.offers.len(), 2);
    assert!(
        result
            .invalid_records
            .iter()
            .any(|e| e.record.source_id == "date" && e.reason.contains("invalid date"))
    );
}
#[test]
fn dedup_is_order_independent_and_keeps_store_and_period_distinct() {
    let a = raw("same");
    let mut duplicate = a.clone();
    duplicate.heading = Some("Andet".into());
    let mut store = a.clone();
    store.store = Some(Store {
        store_id: Some("1".into()),
        name: None,
        location: None,
    });
    let mut period = a.clone();
    period.ends = RawBoundary::Date("2026-11-09".into());
    let mut records = vec![a, duplicate, store, period];
    let first = normalize_batch(metadata(), batch(records.clone()));
    records.reverse();
    let second = normalize_batch(metadata(), batch(records));
    assert_eq!(first, second);
    assert_eq!(first.duplicates, 1);
    assert_eq!(first.offers.len(), 3);
    let mut other = metadata();
    other.id = "anden".into();
    assert_ne!(
        first.offers[0].offer.source,
        normalize_batch(other, batch(vec![raw("same")])).offers[0]
            .offer
            .source
    );
}

#[test]
fn fractional_ore_noise_has_a_small_exact_tolerance() {
    let mut noisy = raw("noise");
    noisy.before = Some("38.760000000000005".into());
    let mut fractional = raw("fractional");
    fractional.amount = Some("1.001".into());
    let result = normalize_batch(metadata(), batch(vec![noisy, fractional]));
    assert_eq!(result.offers.len(), 1);
    assert_eq!(result.offers[0].offer.price.amount_ore, Some(1000));
    assert_eq!(result.offers[0].offer.price.before_ore, Some(3876));
    assert_eq!(result.invalid_records.len(), 1);
}

#[test]
fn unknown_heading_preserves_the_identified_offer() {
    let mut no_heading = raw("untitled");
    no_heading.heading = None;
    let result = normalize_batch(metadata(), batch(vec![no_heading]));
    assert_eq!(result.offers.len(), 1);
    assert_eq!(result.offers[0].offer.heading, None);
    assert!(result.invalid_records.is_empty());
}

#[test]
fn completeness_requires_affirmative_batch_without_errors_or_invalid_records() {
    let mut b = SourceBatch {
        complete: true,
        records: vec![raw("1")],
        ..Default::default()
    };
    assert!(normalize_batch(metadata(), b.clone()).complete);
    b.complete = false;
    assert!(!normalize_batch(metadata(), b.clone()).complete);
    b.complete = true;
    b.errors.push("fetch failed".into());
    assert!(!normalize_batch(metadata(), b.clone()).complete);
    b.errors.clear();
    b.records[0].schema_version = 2;
    assert!(!normalize_batch(metadata(), b).complete);
    assert!(!normalize_batch(metadata(), SourceBatch::default()).complete);
}

#[test]
fn validity_uses_copenhagen_calendar_and_precise_instants() {
    use chrono::{NaiveDate, TimeZone, Utc};
    use tilbud::{
        engine::{ValidityStatus, validity_status},
        offer::{TimeBoundary, Validity},
    };
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 23, 30, 0).unwrap();
    let date = |d| TimeBoundary::Date(NaiveDate::from_ymd_opt(2026, 1, d).unwrap());
    assert_eq!(
        validity_status(
            &Validity {
                starts: date(2),
                ends: date(2)
            },
            now
        ),
        ValidityStatus::Current
    );
    assert_eq!(
        validity_status(
            &Validity {
                starts: date(1),
                ends: date(1)
            },
            now
        ),
        ValidityStatus::Expired
    );
    assert_eq!(
        validity_status(
            &Validity {
                starts: date(3),
                ends: TimeBoundary::Unknown
            },
            now
        ),
        ValidityStatus::Future
    );
    assert_eq!(
        validity_status(
            &Validity {
                starts: TimeBoundary::Unknown,
                ends: date(2)
            },
            now
        ),
        ValidityStatus::Unknown
    );
    let instant =
        |seconds| TimeBoundary::Instant((now + chrono::Duration::seconds(seconds)).fixed_offset());
    assert_eq!(
        validity_status(
            &Validity {
                starts: instant(-1),
                ends: instant(0)
            },
            now
        ),
        ValidityStatus::Unknown
    );
    assert_eq!(
        validity_status(
            &Validity {
                starts: instant(-2),
                ends: instant(-1)
            },
            now
        ),
        ValidityStatus::Expired
    );
}

#[test]
fn exact_instant_boundaries_are_unknown_unless_another_bound_decides() {
    use chrono::{Duration, TimeZone, Utc};
    use tilbud::{
        engine::{ValidityStatus, validity_status},
        offer::{TimeBoundary, Validity},
    };
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
    let instant = |ms| TimeBoundary::Instant((now + Duration::milliseconds(ms)).fixed_offset());
    for (start, end, status) in [
        (0, 1, ValidityStatus::Unknown),
        (-1, 0, ValidityStatus::Unknown),
        (-1, 1, ValidityStatus::Current),
        (1, 2, ValidityStatus::Future),
        (-2, -1, ValidityStatus::Expired),
    ] {
        assert_eq!(
            validity_status(
                &Validity {
                    starts: instant(start),
                    ends: instant(end)
                },
                now
            ),
            status
        );
    }
    // Known date boundaries still decide even if the instant is exactly now.
    let yesterday = TimeBoundary::Date((now - Duration::days(1)).date_naive());
    let tomorrow = TimeBoundary::Date((now + Duration::days(1)).date_naive());
    assert_eq!(
        validity_status(
            &Validity {
                starts: instant(0),
                ends: yesterday
            },
            now
        ),
        ValidityStatus::Expired
    );
    assert_eq!(
        validity_status(
            &Validity {
                starts: tomorrow,
                ends: instant(0)
            },
            now
        ),
        ValidityStatus::Future
    );
}

#[test]
fn major_and_minor_price_units_convert_without_float_or_fractional_ore() {
    use tilbud::source::PriceUnit;
    let mut a = raw("major");
    a.amount = Some("10.35".into());
    let mut b = raw("minor");
    b.amount = Some("1035".into());
    b.price_unit = PriceUnit::Minor;
    let result = normalize_batch(
        metadata(),
        SourceBatch {
            complete: true,
            records: vec![a.clone(), b.clone()],
            ..Default::default()
        },
    );
    assert!(result.complete);
    assert!(
        result
            .offers
            .iter()
            .all(|o| o.offer.price.amount_ore == Some(1035))
    );
    a.amount = Some("10.351".into());
    b.amount = Some("1035.1".into());
    let mut overflow = raw("overflow");
    overflow.amount = Some("18446744073709551616".into());
    overflow.price_unit = PriceUnit::Minor;
    let invalid = normalize_batch(
        metadata(),
        SourceBatch {
            complete: true,
            records: vec![a, b, overflow],
            ..Default::default()
        },
    );
    assert!(!invalid.complete);
    assert_eq!(invalid.invalid_records.len(), 3);
}
