use chrono::{Duration as CDuration, Utc};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};
use tilbud::{
    engine,
    offer::{Membership, PriceKind, Quantity},
    rema::RemaAdapter,
    source::{FetchOptions, SourceAdapter},
};

fn server(responses: Vec<(u16, String)>) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut paths = Vec::new();
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 4096];
            let n = stream.read(&mut buf).unwrap();
            paths.push(
                String::from_utf8_lossy(&buf[..n])
                    .lines()
                    .next()
                    .unwrap()
                    .to_string(),
            );
            write!(stream, "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
        paths
    });
    (url, handle)
}
fn product(id: usize, prices: Vec<Value>, underline: &str) -> Value {
    json!({"id":id,"name":"VARE", "underline":underline,"prices":prices})
}
fn price(value: Value, start: i64, end: i64, offer: bool) -> Value {
    let now = Utc::now();
    json!({"price":value,"is_campaign":offer,"is_advertised":false,"starting_at":(now+CDuration::days(start)).to_rfc3339(),"ending_at":(now+CDuration::days(end)).to_rfc3339(),"max_quantity":null,"deposit":null})
}
fn page(rows: Vec<Value>, page: usize, total: usize) -> (u16, String) {
    (200, json!({"data":rows,"meta":{"pagination":{"current_page":page,"per_page":100,"last_page":total.max(1).div_ceil(100),"total":total}}}).to_string())
}
fn adapter(url: &str) -> RemaAdapter {
    RemaAdapter::with_base(url, Duration::from_secs(2))
}
#[tokio::test(flavor = "current_thread")]
async fn pages_map_current_prices_and_exact_quantities() {
    let mut offer = price(json!(20.23), -1, 1, true);
    offer["max_quantity"] = json!(3);
    offer["price_over_max_quantity"] = json!(31.95);
    offer["deposit"] = json!(1.5);
    let mut a = product(
        0,
        vec![
            price(json!(30), -5, 5, false),
            offer,
            price(json!(40), 2, 20, false),
        ],
        "530 GR. / KLASSISK",
    );
    a["barcodes"] = json!(["5700000000001"]);
    let mut rows = vec![a];
    rows.extend((1..100).map(|i| {
        product(
            i,
            vec![price(json!(6.95), -2, 2, false)],
            if i == 1 {
                "1.98 LTR. / 6 PK."
            } else {
                "200 ML. / TYPE"
            },
        )
    }));
    let (url, task) = server(vec![
        page(rows, 1, 101),
        page(
            vec![product(
                100,
                vec![price(json!(99), -4, -2, true), price(json!(88), 2, 4, true)],
                "CA. 1 KG",
            )],
            2,
            101,
        ),
    ]);
    let result = engine::run(&adapter(&url), &FetchOptions::default()).await;
    let paths = task.join().unwrap();
    assert!(paths[0].contains("page=1") && paths[1].contains("page=2"));
    assert!(
        result.complete,
        "{:?} {:?}",
        result.source_errors, result.invalid_records
    );
    assert_eq!(result.offers.len(), 101);
    let offer = &result
        .offers
        .iter()
        .find(|o| o.offer.source_id == "0")
        .unwrap()
        .offer;
    assert_eq!(offer.price.amount_ore, Some(2023));
    assert_eq!(offer.price.before_ore, None);
    assert_eq!(offer.ean.as_deref(), Some("5700000000001"));
    assert_eq!(offer.product_id.as_deref(), Some("0"));
    assert_eq!(offer.membership, Membership::Unknown);
    assert_eq!(offer.price_kind, PriceKind::Offer);
    assert!(
        matches!(&offer.quantity, Some(Quantity::Exact{amount,unit}) if amount.to_string()=="0.530" && unit=="kg")
    );
    let desc = offer.description.as_deref().unwrap();
    for s in [
        "Maksimumsantal: 3",
        "Pris over maksimum: 31.95",
        "Pant: 1.5",
    ] {
        assert!(desc.contains(s), "{desc}");
    }
    let catalog = &result
        .offers
        .iter()
        .find(|o| o.offer.source_id == "1")
        .unwrap()
        .offer;
    assert_eq!(catalog.price_kind, PriceKind::Catalog);
    assert_eq!(catalog.price.amount_ore, Some(695));
    assert!(
        matches!(&catalog.quantity, Some(Quantity::Exact { amount, unit }) if amount.to_string() == "1.98" && unit == "l")
    );
    let missing = &result
        .offers
        .iter()
        .find(|o| o.offer.source_id == "100")
        .unwrap()
        .offer;
    assert_eq!(missing.price_kind, PriceKind::Catalog);
    assert_eq!(missing.price.amount_ore, None);
    assert!(missing.quantity.is_none());
}
#[tokio::test(flavor = "current_thread")]
async fn ambiguous_price_and_missing_fields_stay_unknown() {
    let p = price(json!(10), -1, 1, true);
    let mut other = p.clone();
    other["price"] = json!(11);
    let (url, task) = server(vec![page(
        vec![product(1, vec![p, other], "omkring 500 g"), json!({"id":2})],
        1,
        2,
    )]);
    let result = engine::run(&adapter(&url), &FetchOptions::default()).await;
    task.join().unwrap();
    assert!(result.complete);
    let a = &result
        .offers
        .iter()
        .find(|x| x.offer.source_id == "1")
        .unwrap()
        .offer;
    assert_eq!(a.price.amount_ore, None);
    assert!(a.description.as_ref().unwrap().contains("modstridende"));
    assert!(a.quantity.is_none());
    let b = &result
        .offers
        .iter()
        .find(|x| x.offer.source_id == "2")
        .unwrap()
        .offer;
    assert_eq!(b.price.amount_ore, None);
    assert_eq!(b.price_kind, PriceKind::Catalog);
}
#[tokio::test(flavor = "current_thread")]
async fn errors_and_unsupported_options_are_incomplete() {
    for (status, body) in [(500, "failure"), (200, "not json"), (200, "{}")] {
        let (url, task) = server(vec![(status, body.into())]);
        let batch = adapter(&url).fetch(&FetchOptions::default()).await;
        task.join().unwrap();
        assert!(!batch.complete);
        assert!(!batch.errors.is_empty());
    }
    let batch = adapter("http://127.0.0.1:1")
        .fetch(&FetchOptions {
            store_id: Some("x".into()),
            date: None,
        })
        .await;
    assert!(!batch.complete);
    assert!(batch.errors[0].contains("understøtter ikke"));
}
#[tokio::test(flavor = "current_thread")]
async fn repeated_page_is_incomplete() {
    let rows = (0..100).map(|i| product(i, vec![], "")).collect::<Vec<_>>();
    let (url, task) = server(vec![page(rows.clone(), 1, 200), page(rows, 2, 200)]);
    let batch = adapter(&url).fetch(&FetchOptions::default()).await;
    task.join().unwrap();
    assert!(!batch.complete);
    assert!(batch.errors.iter().any(|e| e.contains("repeated page")));
}
