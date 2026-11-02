use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};
use tilbud::{
    bilkatogo::BilkaToGoAdapter,
    engine,
    offer::{PriceKind, TimeBoundary},
    source::{FetchOptions, SourceAdapter},
};

fn server(responses: Vec<(u16, String)>) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buf[..n]);
                if let Some(end) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&bytes[..end]);
                    let length = head
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .and_then(|v| v.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            requests.push(String::from_utf8(bytes).unwrap());
            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
        requests
    });
    (url, handle)
}
fn config() -> Vec<(u16, String)> {
    vec![
        (200, "<script src=\"/_nuxt/a.js\"></script>".into()),
        (
            200,
            format!(
                "NUXT_ENV_ALGOLIA_APP_ID:\"ABC123\",NUXT_ENV_ALGOLIA_SEARCH_API_KEY:\"{}\"",
                "a".repeat(32)
            ),
        ),
    ]
}
fn row(id: usize) -> Value {
    json!({"id":id,"name":"Kaffe", "netcontent":"400g", "isInCurrentLeaflet":true, "price":6295,"sales_price":3900,"storeData":{"1653":{"price":3800,"beforePrice":0,"offerDescription":"Skarp pris","offerMax":2},"1651":{"price":5000}}})
}
fn page(hits: Vec<Value>, page: usize, total: usize) -> (u16, String) {
    (200,json!({"hits":hits,"page":page,"nbHits":total,"nbPages":total.div_ceil(1000),"hitsPerPage":1000,"exhaustiveNbHits":true}).to_string())
}
fn adapter(url: &str) -> BilkaToGoAdapter {
    BilkaToGoAdapter::with_endpoints(url, Some(url), Duration::from_secs(3))
}
#[tokio::test(flavor = "current_thread")]
async fn pagination_scope_and_price_are_preserved() {
    let mut responses = config();
    responses.push(page((0..1000).map(row).collect(), 0, 1001));
    responses.push(page(vec![row(1000)], 1, 1001));
    let (url, task) = server(responses);
    let result = engine::run(&adapter(&url), &FetchOptions::default()).await;
    let requests = task.join().unwrap();
    assert!(
        result.complete,
        "{:?} {:?}",
        result.source_errors, result.invalid_records
    );
    assert_eq!(result.offers.len(), 1001);
    let body: Value = serde_json::from_str(requests[2].split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["query"], "");
    assert_eq!(body["page"], 0);
    assert_eq!(
        body["filters"],
        "isInAssortmentIn:1653 AND inStockStore:1653 AND nonsearchable:false AND isInCurrentLeaflet:true"
    );
    assert_eq!(body["hitsPerPage"], 1000);
    assert_eq!(
        serde_json::from_str::<Value>(requests[3].split("\r\n\r\n").nth(1).unwrap()).unwrap()["page"],
        1
    );
    let offer = &result.offers[0].offer;
    assert_eq!(offer.price.amount_ore, Some(3800));
    assert_eq!(offer.price.before_ore, None);
    assert_eq!(
        offer.store.as_ref().unwrap().store_id.as_deref(),
        Some("1653")
    );
    assert_eq!(offer.price_kind, PriceKind::Offer);
    assert_eq!(offer.validity.starts, TimeBoundary::Unknown);
    assert!(offer.description.as_ref().unwrap().contains("Skarp pris"));
    assert_eq!(
        result.offers[0]
            .unit_price
            .as_ref()
            .unwrap()
            .ore
            .to_string(),
        "9500"
    );
}
#[tokio::test(flavor = "current_thread")]
async fn missing_store_never_uses_root_or_other_store_and_ambiguous_quantity_is_retained() {
    let mut product = row(1);
    product["storeData"]["1653"] = Value::Null;
    product["netcontent"] = json!("2 x 400 g");
    product["isInCurrentLeaflet"] = json!(false);
    let mut responses = config();
    responses.push(page(vec![product], 0, 1));
    let (url, task) = server(responses);
    let result = engine::run(&adapter(&url), &FetchOptions::default()).await;
    task.join().unwrap();
    assert!(result.complete);
    let o = &result.offers[0].offer;
    assert_eq!(o.price.amount_ore, None);
    assert_eq!(o.price.before_ore, None);
    assert_eq!(o.price_kind, PriceKind::Catalog);
    assert_eq!(o.quantity, None);
    assert!(o.description.as_ref().unwrap().contains("2 x 400 g"));
}
#[tokio::test(flavor = "current_thread")]
async fn refresh_once_on_rejection_and_stop_on_second_rejection() {
    for status in [401, 403] {
        let mut responses = config();
        responses.push((status, "".into()));
        responses.extend(config());
        responses.push((status, "".into()));
        let (url, task) = server(responses);
        let result = adapter(&url).fetch(&FetchOptions::default()).await;
        let requests = task.join().unwrap();
        assert!(!result.complete);
        assert_eq!(requests.len(), 6);
        assert!(result.errors[0].contains(&status.to_string()));
    }
    let mut responses = config();
    responses.push((401, "".into()));
    responses.extend(config());
    responses.push(page(vec![row(1)], 0, 1));
    let (url, task) = server(responses);
    let result = adapter(&url).fetch(&FetchOptions::default()).await;
    task.join().unwrap();
    assert!(result.complete);
}
#[tokio::test(flavor = "current_thread")]
async fn rejects_drift_duplicates_truncation_and_malformed_data() {
    let cases = vec![
        vec![page((0..1000).map(row).collect(),0,1001),page(vec![row(0)],1,1001)],
        vec![page((0..1000).map(row).collect(),0,1001),page(vec![row(1000),row(1001)],1,1002)],
        vec![(200,json!({"hits":[],"page":0,"nbHits":1000,"nbPages":1,"hitsPerPage":1000,"exhaustiveNbHits":false}).to_string())],
        vec![page(vec![json!({"name":"no id"})],0,1)],
        vec![(200,"{}".into())],
    ];
    for pages in cases {
        let mut responses = config();
        responses.extend(pages);
        let (url, task) = server(responses);
        let result = adapter(&url).fetch(&FetchOptions::default()).await;
        task.join().unwrap();
        assert!(!result.complete);
        assert!(!result.errors.is_empty());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn selected_store_drives_filter_and_price_and_invalid_options_fail_closed() {
    let mut responses = config();
    responses.push(page(vec![row(3)], 0, 1));
    let (url, task) = server(responses);
    let result = engine::run(
        &adapter(&url),
        &FetchOptions {
            store_id: Some("1651".into()),
            date: None,
        },
    )
    .await;
    let requests = task.join().unwrap();
    assert!(result.complete);
    let body: Value = serde_json::from_str(requests[2].split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert!(
        body["filters"]
            .as_str()
            .unwrap()
            .contains("isInAssortmentIn:1651 AND inStockStore:1651")
    );
    assert_eq!(result.offers[0].offer.price.amount_ore, Some(5000));
    assert_eq!(
        result.offers[0]
            .offer
            .store
            .as_ref()
            .unwrap()
            .store_id
            .as_deref(),
        Some("1651")
    );
    assert!(result.coverage.unwrap().contains("1651"));
    let batch = adapter("http://127.0.0.1:1")
        .fetch(&FetchOptions {
            store_id: Some("1653 OR true".into()),
            date: None,
        })
        .await;
    assert!(!batch.complete);
}

#[tokio::test(flavor = "current_thread")]
async fn missing_config_and_invalid_selected_price_fail_without_snapshot() {
    let (url, task) = server(vec![
        (200, "<script src='/_nuxt/a.js'></script>".into()),
        (200, "no configuration".into()),
    ]);
    let batch = adapter(&url).fetch(&FetchOptions::default()).await;
    task.join().unwrap();
    assert!(!batch.complete);
    assert!(batch.errors[0].contains("configuration missing"));
    let mut bad = row(1);
    bad["storeData"]["1653"]["price"] = json!({"unexpected":true});
    let mut responses = config();
    responses.push(page(vec![bad], 0, 1));
    let (url, task) = server(responses);
    let batch = adapter(&url).fetch(&FetchOptions::default()).await;
    task.join().unwrap();
    assert!(!batch.complete);
    assert!(batch.errors[0].contains("invalid selected-store price"));
}
