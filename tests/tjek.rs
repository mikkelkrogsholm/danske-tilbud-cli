use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};
use tilbud::{
    engine,
    offer::{Membership, PieceCount, Quantity, QuantityBasis},
    source::{FetchOptions, SourceAdapter},
    tjek::TjekAdapter,
};

fn row(id: &str) -> Value {
    json!({"id":id,"dealer_id":"11deC","catalog_id":"catalog1","dealer":{"id":"11deC","country":{"id":"DK"}},"heading":"Sodavand","description":"Maks. 6; derefter 20,53 kr.","pricing":{"price":10.23,"pre_price":20.53,"currency":"DKK"},"quantity":{"unit":{"symbol":"l"},"size":{"from":1.5,"to":1.5},"pieces":{"from":2,"to":2,"max":6}},"run_from":"2026-10-31T23:00:00+0000","run_till":"2026-11-07T22:59:59+0000","store_id":null})
}
fn fixture_server(
    responses: Vec<(u16, String, String, u64)>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    raw_server(responses)
}
fn raw_server(
    responses: Vec<(u16, String, String, u64)>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v2", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut paths = Vec::new();
        for (status, headers, body, delay) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 4096];
            let n = stream.read(&mut buf).unwrap();
            paths.push(
                String::from_utf8_lossy(&buf[..n])
                    .lines()
                    .next()
                    .unwrap()
                    .to_owned(),
            );
            if delay > 0 {
                thread::sleep(Duration::from_millis(delay));
            }
            let reply = format!(
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n{headers}\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
        paths
    });
    (url, handle)
}
fn ok(rows: Vec<Value>) -> (u16, String, String, u64) {
    (200, String::new(), serde_json::to_string(&rows).unwrap(), 0)
}
fn capped_prefix() -> Vec<(u16, String, String, u64)> {
    let mut responses: Vec<_> = (0..10)
        .map(|page| ok((0..100).map(|i| row(&format!("{page}-{i}"))).collect()))
        .collect();
    responses.push((
        400,
        String::new(),
        r#"{"name":"PAGINATION_MAX_REACHED"}"#.into(),
        0,
    ));
    responses
}
fn adapter(url: &str, ms: u64) -> TjekAdapter {
    TjekAdapter::with_base(url, Duration::from_millis(ms))
        .select_chains(&["rema1000"])
        .unwrap()
}
#[tokio::test(flavor = "current_thread")]
async fn pagination_and_engine_preserve_exact_source_values() {
    let first: Vec<_> = (0..100).map(|i| row(&format!("id{i}"))).collect();
    let (url, server) = fixture_server(vec![ok(first), ok(vec![row("last")])]);
    let result = engine::run(&adapter(&url, 1000), &FetchOptions::default()).await;
    let paths = server.join().unwrap();
    assert!(paths[0].contains("dealer_id=11deC") && paths[1].contains("offset=100"));
    assert_eq!(result.offers.len(), 101);
    assert!(result.source_errors.is_empty());
    assert!(result.invalid_records.is_empty());
    let offer = &result.offers[0].offer;
    assert_eq!(offer.price.amount_ore, Some(1023));
    assert_eq!(offer.price.before_ore, Some(2053));
    assert_eq!(offer.membership, Membership::Unknown);
    assert_eq!(offer.pieces, Some(PieceCount::Exact { count: 2 }));
    assert_eq!(offer.quantity_basis, Some(QuantityBasis::PerPiece));
    assert!(offer.description.as_ref().unwrap().contains("Maks. 6"));
    assert!(
        offer
            .description
            .as_ref()
            .unwrap()
            .contains("maksimumsgrænse: 6 stk.")
    );
}
#[tokio::test(flavor = "current_thread")]
async fn filtering_store_date_and_country() {
    let mut local = row("local");
    local["store_id"] = json!("other");
    let (url, server) = fixture_server(vec![ok(vec![row("national"), local])]);
    let options = FetchOptions {
        store_id: Some("chosen".into()),
        date: Some(chrono::NaiveDate::from_ymd_opt(2026, 11, 1).unwrap()),
    };
    let batch = adapter(&url, 1000).fetch(&options).await;
    server.join().unwrap();
    assert_eq!(batch.records.len(), 1);
    assert!(batch.errors.is_empty());
    let mut foreign = row("foreign");
    foreign["dealer"]["country"]["id"] = json!("SE");
    let (url, server) = fixture_server(vec![ok(vec![foreign])]);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(batch.records.is_empty());
    assert!(batch.errors[0].contains("wrong dealer or country"));
}
#[tokio::test(flavor = "current_thread")]
async fn retry_after_and_bad_responses_are_visible() {
    let (url, server) = fixture_server(vec![
        (429, "Retry-After: 0\r\n".into(), "[]".into(), 0),
        ok(vec![row("one")]),
    ]);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    assert_eq!(server.join().unwrap().len(), 2);
    assert_eq!(batch.records.len(), 1);
    let (url, server) = fixture_server(vec![(429, "Retry-After: 60\r\n".into(), "[]".into(), 0)]);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(batch.errors[0].contains("too long"));
    let (url, server) = fixture_server(vec![(200, String::new(), "{}".into(), 0)]);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(batch.errors[0].contains("invalid response"));
    let mut broken = row("bad");
    broken["pricing"]["price"] = json!("10.23");
    let (url, server) = fixture_server(vec![ok(vec![broken])]);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(batch.errors[0].contains("invalid price"));
}
#[tokio::test(flavor = "current_thread")]
async fn timeouts_and_repeated_pages_are_partial_errors() {
    let (url, server) = fixture_server(vec![
        (200, String::new(), "[]".into(), 100),
        (200, String::new(), "[]".into(), 100),
        (200, String::new(), "[]".into(), 100),
    ]);
    let batch = adapter(&url, 30).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(!batch.errors.is_empty());
    let rows: Vec<_> = (0..100).map(|i| row(&format!("{i}"))).collect();
    let (url, server) = fixture_server(vec![ok(rows.clone()), ok(rows)]);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(batch.errors[0].contains("repeated page"));
    assert!(batch.coverage.unwrap().contains("partial"));
}
#[tokio::test(flavor = "current_thread")]
async fn pcs_is_normalized_by_engine() {
    let mut r = row("count");
    r["quantity"]["unit"]["symbol"] = json!("pcs");
    let (url, server) = fixture_server(vec![ok(vec![r])]);
    let result = engine::run(&adapter(&url, 1000), &FetchOptions::default()).await;
    server.join().unwrap();
    assert!(
        matches!(&result.offers[0].offer.quantity, Some(Quantity::Exact { unit, .. }) if unit == "stk")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn bad_row_is_reported_but_following_offer_is_kept() {
    let mut bad = row("blank");
    bad["heading"] = json!("");
    let mut wrong_catalog = row("wrong");
    wrong_catalog["dealer_id"] = json!("other");
    let (url, server) = fixture_server(vec![ok(vec![bad, wrong_catalog, row("good")])]);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert_eq!(batch.records.len(), 2);
    assert_eq!(batch.errors.len(), 1);
    assert!(batch.records.iter().any(|r| r.heading.is_none()));
    assert!(batch.coverage.unwrap().contains("partial"));
}

#[tokio::test(flavor = "current_thread")]
async fn catalog_pages_are_traversed_before_offer_pages() {
    let catalog = |id: &str| json!({"id":id, "dealer_id":"11deC", "dealer":{"id":"11deC","country":{"id":"DK"}}, "store_id":null});
    let first: Vec<_> = (0..100).map(|i| catalog(&format!("c{i}"))).collect();
    let mut responses = capped_prefix();
    responses.extend(vec![ok(first), ok(vec![catalog("catalog1")])]);
    responses.extend((0..100).map(|_| ok(vec![])));
    responses.push(ok(vec![row("last")]));
    let (url, server) = raw_server(responses);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    let paths = server.join().unwrap();
    assert!(paths[11].contains("/catalogs?") && paths[12].contains("offset=100"));
    assert_eq!(batch.records.len(), 1);
    assert!(batch.errors.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn empty_catalogs_are_not_claimed_as_offer_coverage() {
    let mut responses = capped_prefix();
    responses.push(ok(vec![]));
    let (url, server) = raw_server(responses);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(batch.errors[0].contains("coverage unknown"));
    assert!(batch.coverage.unwrap().contains("partial"));
}

#[tokio::test(flavor = "current_thread")]
async fn catalog_store_is_preserved_and_dates_filter_in_copenhagen() {
    let catalog = json!({"id":"catalog1", "dealer_id":"11deC", "dealer":{"id":"11deC","country":{"id":"DK"}}, "store_id":"local"});
    let mut responses = capped_prefix();
    responses.extend(vec![ok(vec![catalog]), ok(vec![row("local")])]);
    let (url, server) = raw_server(responses);
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert_eq!(
        batch.records[0].store.as_ref().unwrap().store_id.as_deref(),
        Some("local")
    );
    let (url, server) = fixture_server(vec![ok(vec![row("outside")])]);
    let options = FetchOptions {
        store_id: None,
        date: Some(chrono::NaiveDate::from_ymd_opt(2026, 10, 31).unwrap()),
    };
    let batch = adapter(&url, 1000).fetch(&options).await;
    server.join().unwrap();
    assert!(batch.records.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn retry_after_is_not_retried_early() {
    let (url, server) = fixture_server(vec![
        (429, "Retry-After: 1\r\n".into(), "[]".into(), 0),
        ok(vec![row("one")]),
    ]);
    let start = std::time::Instant::now();
    let batch = adapter(&url, 1000).fetch(&FetchOptions::default()).await;
    server.join().unwrap();
    assert!(start.elapsed() >= Duration::from_secs(1));
    assert!(batch.errors.is_empty());
}
