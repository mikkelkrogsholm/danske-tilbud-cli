use tilbud::offer::{Membership, Offer, Quantity, TimeBoundary};

#[test]
fn documented_examples_round_trip_without_losing_precision_or_unknowns() {
    let document = include_str!("../docs/offer-v1.md");
    let examples: Vec<_> = document
        .split("```json\n")
        .skip(1)
        .map(|part| part.split("\n```").next().unwrap())
        .collect();
    assert_eq!(examples.len(), 2);
    for example in examples {
        let original: serde_json::Value = serde_json::from_str(example).unwrap();
        let offer: Offer = serde_json::from_value(original.clone()).unwrap();
        let encoded = serde_json::to_value(&offer).unwrap();
        // UTC may be rendered as +00:00 rather than Z, but means the same instant.
        let decoded: Offer = serde_json::from_value(encoded).unwrap();
        assert_eq!(offer, decoded);
        assert_eq!(offer.schema_version, 1);
        if offer.source_id == "123" {
            assert_eq!(offer.membership, Membership::Unknown);
            assert_eq!(offer.validity.ends, TimeBoundary::Unknown);
            assert!(matches!(offer.quantity, Some(Quantity::Exact { .. })));
            assert_eq!(offer.price.before_ore, Some(2053));
            assert_eq!(offer.ean.as_deref(), Some("0123456789012"));
        } else {
            assert!(matches!(offer.validity.starts, TimeBoundary::Date(_)));
            assert!(matches!(offer.quantity, Some(Quantity::Range { .. })));
        }
    }
}

#[test]
fn unknown_membership_is_distinct_from_required_and_not_required() {
    let unknown = serde_json::to_string(&Membership::Unknown).unwrap();
    assert_ne!(
        unknown,
        serde_json::to_string(&Membership::Required).unwrap()
    );
    assert_ne!(
        unknown,
        serde_json::to_string(&Membership::NotRequired).unwrap()
    );
    assert_eq!(
        serde_json::from_str::<Membership>(&unknown).unwrap(),
        Membership::Unknown
    );
}

#[test]
fn unknown_price_and_package_count_round_trip_without_becoming_zero() {
    use tilbud::offer::{PieceCount, Price, QuantityBasis};

    let unknown: Price = serde_json::from_str(r#"{"amount_ore":null,"before_ore":null}"#).unwrap();
    let free: Price = serde_json::from_str(r#"{"amount_ore":0,"before_ore":null}"#).unwrap();
    assert_ne!(unknown, free);
    assert_eq!(
        serde_json::to_value(&unknown).unwrap()["amount_ore"],
        serde_json::Value::Null
    );
    assert_eq!(serde_json::to_value(&free).unwrap()["amount_ore"], 0);

    let example = include_str!("../docs/offer-v1.md")
        .split("```json\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(example).unwrap();
    value["quantity"] = serde_json::json!({"kind": "exact", "amount": "330", "unit": "ml"});
    value["quantity_basis"] = serde_json::json!("per_piece");
    value["pieces"] = serde_json::json!({"kind": "exact", "count": 6});
    value["price"]["amount_ore"] = serde_json::Value::Null;
    let offer: Offer = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(offer.quantity_basis, Some(QuantityBasis::PerPiece));
    assert_eq!(offer.pieces, Some(PieceCount::Exact { count: 6 }));
    assert_eq!(serde_json::to_value(&offer).unwrap(), value);

    value["pieces"] = serde_json::Value::Null;
    let unknown_count: Offer = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(unknown_count.pieces, None);
    assert_eq!(unknown_count.quantity_basis, Some(QuantityBasis::PerPiece));
    assert_eq!(serde_json::to_value(&unknown_count).unwrap(), value);

    value["pieces"] = serde_json::json!({"kind": "range", "min": 4, "max": 6});
    let range: Offer = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(range.pieces, Some(PieceCount::Range { min: 4, max: 6 }));
    assert_eq!(serde_json::to_value(range).unwrap(), value);
}
