use std::process::Command;

#[test]
fn help_and_invalid_arguments_are_handled_by_the_parser() {
    let executable = env!("CARGO_BIN_EXE_tilbud");
    let help = Command::new(executable).arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("tilbud"));
    assert!(text.contains("--help"));
    let invalid = Command::new(executable).arg("--ukendt").output().unwrap();
    assert!(!invalid.status.success());
}

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::var_os("TILBUD_TEST_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!(
            "tilbud-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_tilbud"))
            .args(args)
            .env("TILBUD_DATA_DIR", &self.0)
            .output()
            .unwrap()
    }
    fn seed(&self) {
        let example = include_str!("../docs/offer-v1.md")
            .split("```json\n")
            .nth(1)
            .unwrap()
            .split("\n```")
            .next()
            .unwrap();
        let mut base: Value = serde_json::from_str(example).unwrap();
        base["source"] = json!("tjek");
        base["chain_id"] = json!("netto");
        base["heading"] = json!("ØKO Kaffe");
        base["price_kind"] = json!("offer");
        let today = Utc::now()
            .with_timezone(&chrono_tz::Europe::Copenhagen)
            .date_naive();
        let date =
            |days| json!({"precision":"date", "value":(today + Duration::days(days)).to_string()});
        base["validity"] = json!({"starts":date(-1),"ends":date(1)});
        let mut rows = Vec::new();
        for (id, chain, kind, start, end) in [
            ("current", "netto", "offer", date(-1), date(1)),
            ("other", "bilka", "offer", date(-1), date(1)),
            ("catalog", "netto", "catalog", date(-1), date(1)),
            ("past", "netto", "offer", date(-3), date(-1)),
            ("future", "netto", "offer", date(1), date(3)),
            (
                "unknown",
                "netto",
                "offer",
                json!({"precision":"unknown"}),
                date(1),
            ),
        ] {
            let mut row = base.clone();
            row["source_id"] = json!(id);
            row["chain_id"] = json!(chain);
            row["price_kind"] = json!(kind);
            row["validity"] = json!({"starts":start,"ends":end});
            rows.push(row);
        }
        let manifest = json!({"record_type":"snapshot","schema_version":1,"source":"tjek","fetched_at":Utc::now().to_rfc3339(),"written_at":Utc::now().to_rfc3339(),"coverage":null,"complete":true,"offer_count":rows.len()});
        let mut contents = format!("{manifest}\n");
        for row in rows {
            contents.push_str(&format!("{row}\n"));
        }
        fs::write(self.0.join("tjek.jsonl"), contents).unwrap();
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn success(out: std::process::Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stderr.is_empty());
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn local_search_filters_and_reports_snapshot() {
    let dir = Temp::new();
    dir.seed();
    let sources = success(dir.run(&["kilder"]));
    assert_eq!(sources["data"]["sources"][0]["snapshot"]["offer_count"], 6);
    let described = success(dir.run(&["beskriv", "tjek"]));
    assert_eq!(described["data"]["id"], "tjek");
    let found = success(dir.run(&["find", "øko", "--butik", "netto", "--kilde", "tjek"]));
    assert_eq!(found["schema_version"], 1);
    assert_eq!(found["data"]["total"], 1);
    assert_eq!(found["data"]["results"][0]["offer"]["source_id"], "current");
    let unknown = success(dir.run(&["find", "KAFFE", "--butik", "netto", "--include-unknown"]));
    assert_eq!(unknown["data"]["total"], 2);
    assert!(
        unknown["data"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["validity_status"] == "unknown")
    );
    let compact_unknown = success(dir.run(&[
        "find",
        "KAFFE",
        "--butik",
        "netto",
        "--include-unknown",
        "--compact",
    ]));
    let compact_rows = compact_unknown["data"]["results"].as_array().unwrap();
    let full_rows = unknown["data"]["results"].as_array().unwrap();
    for (compact, full) in compact_rows.iter().zip(full_rows) {
        assert_eq!(compact["source_id"], full["offer"]["source_id"]);
        assert_eq!(compact["membership"], full["offer"]["membership"]);
        assert_eq!(compact["validity"], full["offer"]["validity"]);
        assert_eq!(compact["validity_status"], full["validity_status"]);
    }
    let limited = success(dir.run(&["find", "kaffe", "--limit", "1"]));
    assert_eq!(limited["data"]["total"], 2);
    assert_eq!(limited["data"]["results"].as_array().unwrap().len(), 1);
    for args in [
        &["find", "kaffe", "--kilde", "bogus"][..],
        &["find", "kaffe", "--butik", "bogus"],
        &["find", "kaffe", "--limit", "0"],
        &["find", "kaffe", "--limit", "x"],
    ] {
        let out = dir.run(args);
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
    }
}
#[test]
fn missing_and_corrupt_snapshots_fail_usefully() {
    let dir = Temp::new();
    assert!(success(dir.run(&["kilder"]))["data"]["sources"][0]["snapshot"].is_null());
    for args in [&["hent"][..], &["hent", "--alle", "--kilde", "tjek"][..]] {
        let invalid = dir.run(args);
        assert!(!invalid.status.success());
        assert!(String::from_utf8_lossy(&invalid.stderr).contains("præcis én"));
    }
    let missing = dir.run(&["find", "kaffe"]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("hent"));
    fs::write(dir.0.join("tjek.jsonl"), "broken\n").unwrap();
    let corrupt = dir.run(&["find", "kaffe"]);
    assert!(!corrupt.status.success());
    assert!(String::from_utf8_lossy(&corrupt.stderr).contains("snapshot"));
}

#[test]
fn human_uses_same_filtered_results_and_is_plain_when_piped() {
    let dir = Temp::new();
    dir.seed();
    let json_out = dir.run(&["find", "kaffe", "--butik", "netto", "--include-unknown"]);
    assert!(!json_out.stdout.contains(&27));
    let json = success(json_out);
    let human = Command::new(env!("CARGO_BIN_EXE_tilbud"))
        .args([
            "find",
            "kaffe",
            "--butik",
            "netto",
            "--include-unknown",
            "--human",
        ])
        .env("TILBUD_DATA_DIR", &dir.0)
        .env("NO_COLOR", "1")
        .env("COLUMNS", "32")
        .output()
        .unwrap();
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    assert!(!human.stdout.contains(&27));
    let text = String::from_utf8(human.stdout).unwrap();
    assert_eq!(
        text.matches("ØKO Kaffe").count(),
        json["data"]["results"].as_array().unwrap().len()
    );
    assert!(text.contains("Medlemskab:"));
    assert!(text.contains("Gyldig:"));
    assert!(text.contains("ukendt"));
    assert!(text.contains("Kildedata hentet:"));
    assert!(!text.contains("normalpris"));
    assert!(!text.contains("besparelse"));
    let empty = dir.run(&["find", "findes-ikke", "--human"]);
    assert!(empty.status.success());
    assert!(String::from_utf8_lossy(&empty.stdout).contains("Ingen tilbud fundet"));
    let described = dir.run(&["beskriv", "tjek", "--human"]);
    let description = String::from_utf8(described.stdout).unwrap();
    assert!(description.contains("Dækning:"));
    assert!(description.contains("Bilka"));
    let missing = dir.run(&["find", "kaffe", "--kilde", "ukendt", "--human"]);
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    assert!(!missing.stderr.contains(&27));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("Ukendt kilde"));
}

#[test]
fn human_narrow_search_keeps_long_names_conditions_and_unknown_prices() {
    let dir = Temp::new();
    dir.seed();
    let path = dir.0.join("tjek.jsonl");
    let contents = fs::read_to_string(&path).unwrap();
    let mut rows: Vec<Value> = contents
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    rows[1]["heading"] = json!("Økologisk jordbærmarmelade med ekstra lange danske navne");
    rows[1]["description"] = json!("Gælder kun ved køb af to varer og særlige medlemsbetingelser");
    rows[1]["membership"] = json!("required");
    rows[1]["price"]["amount_ore"] = Value::Null;
    fs::write(
        &path,
        rows.iter().map(|r| format!("{r}\n")).collect::<String>(),
    )
    .unwrap();
    let human = Command::new(env!("CARGO_BIN_EXE_tilbud"))
        .args(["find", "jordbær", "--human"])
        .env("TILBUD_DATA_DIR", &dir.0)
        .env("COLUMNS", "32")
        .output()
        .unwrap();
    assert!(human.status.success());
    assert!(!human.stdout.contains(&27));
    let text = String::from_utf8(human.stdout).unwrap();
    for term in [
        "Økologisk",
        "jordbærmarmelade",
        "lange danske navne",
        "Gælder kun ved køb",
        "medlemsbetingelser",
        "Medlemskab: kræves",
        "ukendt pris",
    ] {
        assert!(text.contains(term), "{text}");
    }
}

#[test]
fn rema_and_tjek_snapshots_coexist_and_catalog_does_not_leak_into_search() {
    let dir = Temp::new();
    dir.seed();
    let tjek_before = fs::read(dir.0.join("tjek.jsonl")).unwrap();
    let rows = String::from_utf8(tjek_before.clone()).unwrap();
    let mut lines = rows.lines();
    let mut manifest: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    let first: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    manifest["source"] = json!("rema");
    manifest["offer_count"] = json!(2);
    let mut offer = first.clone();
    offer["source"] = json!("rema");
    offer["chain_id"] = json!("rema1000");
    offer["source_id"] = json!("rema-offer");
    offer["product_id"] = json!("rema-offer");
    let mut catalog = offer.clone();
    catalog["source_id"] = json!("rema-catalog");
    catalog["price_kind"] = json!("catalog");
    fs::write(
        dir.0.join("rema.jsonl"),
        format!("{manifest}\n{offer}\n{catalog}\n"),
    )
    .unwrap();
    let sources = success(dir.run(&["kilder"]));
    assert_eq!(sources["data"]["sources"].as_array().unwrap().len(), 3);
    let rema = success(dir.run(&["beskriv", "rema"]));
    assert_eq!(rema["data"]["snapshot"]["promotion_count"], 1);
    assert_eq!(rema["data"]["snapshot"]["catalog_count"], 1);
    let found = success(dir.run(&["find", "kaffe", "--kilde", "rema"]));
    assert_eq!(found["data"]["total"], 1);
    assert_eq!(
        found["data"]["results"][0]["offer"]["source_id"],
        "rema-offer"
    );
    let human = dir.run(&["beskriv", "rema", "--human"]);
    assert!(human.status.success());
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.contains("2 poster"));
    assert!(text.contains("1 tilbud"));
    assert_eq!(fs::read(dir.0.join("tjek.jsonl")).unwrap(), tjek_before);
}

#[test]
fn combined_search_preserves_sources_and_reports_partial_failure() {
    let dir = Temp::new();
    dir.seed();
    let text = fs::read_to_string(dir.0.join("tjek.jsonl")).unwrap();
    let mut lines = text.lines();
    let manifest: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    let base: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    let mut tjek_lines: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    tjek_lines[1]["ean"] = json!("1234567890123");
    fs::write(
        dir.0.join("tjek.jsonl"),
        tjek_lines
            .iter()
            .map(|r| format!("{r}\n"))
            .collect::<String>(),
    )
    .unwrap();
    for (id, chain, membership) in [
        ("rema", "rema1000", "required"),
        ("bilkatogo", "bilka", "not_required"),
    ] {
        let mut m = manifest.clone();
        m["source"] = json!(id);
        m["offer_count"] = json!(1);
        let mut o = base.clone();
        o["source"] = json!(id);
        o["chain_id"] = json!(chain);
        o["source_id"] = json!("same-id");
        o["ean"] = json!("1234567890123");
        o["membership"] = json!(membership);
        if id == "bilkatogo" {
            let earlier = Utc::now()
                .with_timezone(&chrono_tz::Europe::Copenhagen)
                .date_naive()
                - Duration::days(2);
            o["validity"]["starts"]["value"] = json!(earlier.to_string());
        }
        o["store"] = json!({"store_id":id,"name":id,"location":"Danmark"});
        fs::write(dir.0.join(format!("{id}.jsonl")), format!("{m}\n{o}\n")).unwrap();
    }
    let found = success(dir.run(&["find", "kaffe"]));
    assert_eq!(found["data"]["total"], 4);
    let sources = found["data"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["offer"]["source"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(sources.len(), 3);
    let matched: Vec<_> = found["data"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["offer"]["ean"] == "1234567890123")
        .collect();
    assert_eq!(matched.len(), 3);
    assert_ne!(
        matched
            .iter()
            .find(|r| r["offer"]["source"] == "bilkatogo")
            .unwrap()["offer"]["validity"],
        matched
            .iter()
            .find(|r| r["offer"]["source"] == "rema")
            .unwrap()["offer"]["validity"]
    );
    assert!(matched.iter().any(
        |r| r["offer"]["membership"] == "required" && r["offer"]["store"]["store_id"] == "rema"
    ));
    assert!(
        matched
            .iter()
            .any(|r| r["offer"]["membership"] == "not_required"
                && r["offer"]["store"]["store_id"] == "bilkatogo")
    );
    assert_eq!(
        success(dir.run(&["find", "kaffe", "--butik", "rema1000"]))["data"]["total"],
        1
    );
    assert_eq!(
        success(dir.run(&["find", "kaffe", "--kilde", "bilkatogo"]))["data"]["total"],
        1
    );
    assert_eq!(
        success(dir.run(&["find", "kaffe", "--limit", "2"]))["data"]["results"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    fs::write(dir.0.join("rema.jsonl"), "broken\n").unwrap();
    let scoped = success(dir.run(&["find", "kaffe", "--butik", "netto"]));
    assert_eq!(scoped["data"]["total"], 1);
    assert_eq!(scoped["data"]["complete"], true);
    assert!(
        !scoped["data"]["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["source"] == "rema")
    );
    let invalid = dir.run(&["find", "kaffe", "--kilde", "rema", "--butik", "netto"]);
    assert!(!invalid.status.success());
    let out = dir.run(&["find", "kaffe"]);
    assert!(!out.status.success());
    assert!(!out.stderr.is_empty());
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["error"], "partial_result");
    assert_eq!(value["data"]["total"], 3);
    assert_eq!(value["data"]["sources"][1]["status"], "failed");
    let compact = dir.run(&[
        "find",
        "kaffe",
        "--compact",
        "--limit",
        "1",
        "--offset",
        "1",
    ]);
    assert!(!compact.status.success());
    let compact: Value = serde_json::from_slice(&compact.stdout).unwrap();
    assert_eq!(compact["error"], "partial_result");
    assert_eq!(compact["data"]["total"], 3);
    assert_eq!(compact["data"]["offset"], 1);
    assert_eq!(compact["data"]["has_more"], true);
    assert_eq!(
        compact["data"]["results"][0]["source_id"],
        value["data"]["results"][1]["offer"]["source_id"]
    );
    let human = dir.run(&["find", "kaffe", "--human"]);
    assert!(!human.status.success());
    let text = String::from_utf8_lossy(&human.stdout);
    assert!(text.contains("rema"));
    assert!(text.contains("Søgning:"));
    assert!(!text.trim_start().starts_with('{'));
    assert!(!human.stderr.is_empty());
}

#[tokio::test]
async fn offline_fetch_pipeline_publishes_files_consumed_by_real_cli() {
    use tilbud::{
        engine,
        offer::{Membership, PriceKind},
        source::{
            FetchOptions, PriceUnit, RawBoundary, RawOffer, SourceAdapter, SourceBatch,
            SourceCapabilities, SourceMetadata,
        },
        storage,
    };
    struct Offline {
        metadata: SourceMetadata,
        batch: SourceBatch,
    }
    impl SourceAdapter for Offline {
        fn metadata(&self) -> &SourceMetadata {
            &self.metadata
        }
        async fn fetch(&self, _: &FetchOptions) -> SourceBatch {
            self.batch.clone()
        }
    }
    let dir = Temp::new();
    let now = Utc::now();
    let today = now
        .with_timezone(&chrono_tz::Europe::Copenhagen)
        .date_naive();
    let raw = |id: &str, end| RawOffer {
        schema_version: 1,
        source_id: id.into(),
        chain_id: "netto".into(),
        store: None,
        heading: Some("Kaffe".into()),
        description: None,
        currency: "DKK".into(),
        price_unit: PriceUnit::Major,
        amount: Some("10".into()),
        before: None,
        quantity: None,
        quantity_basis: None,
        pieces: None,
        starts: RawBoundary::Date((today - Duration::days(5)).to_string()),
        ends: end,
        membership: Membership::Unknown,
        fetched_at: now.to_rfc3339(),
        source_reference: "test".into(),
        ean: None,
        product_id: None,
        price_kind: PriceKind::Offer,
    };
    let adapter = Offline {
        metadata: SourceMetadata {
            id: "tjek".into(),
            name: "test".into(),
            chains: vec!["netto".into()],
            coverage_limitation: "test".into(),
            capabilities: SourceCapabilities::default(),
        },
        batch: SourceBatch {
            complete: true,
            records: vec![
                raw(
                    "expired",
                    RawBoundary::Date((today - Duration::days(1)).to_string()),
                ),
                raw(
                    "current",
                    RawBoundary::Date((today + Duration::days(1)).to_string()),
                ),
            ],
            ..Default::default()
        },
    };
    let mut run = engine::run(&adapter, &FetchOptions::default()).await;
    assert!(run.complete);
    assert_eq!(engine::remove_expired(&mut run, now), 1);
    storage::publish(&dir.0, &run, now.fixed_offset()).unwrap();
    let bytes = fs::read_to_string(dir.0.join("tjek.jsonl")).unwrap();
    assert!(!bytes.contains("expired"));
    assert!(bytes.contains("current"));
    let result = success(dir.run(&["find", "kaffe", "--kilde", "tjek"]));
    assert_eq!(result["data"]["total"], 1);
    assert_eq!(
        result["data"]["results"][0]["offer"]["source_id"],
        "current"
    );
}

#[test]
fn agent_search_ranks_pages_and_preserves_compact_terms() {
    let dir = Temp::new();
    dir.seed();
    let text = fs::read_to_string(dir.0.join("tjek.jsonl")).unwrap();
    let mut lines = text.lines();
    let mut manifest: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    let base: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    let mut rows = Vec::new();
    for (id, heading, description) in [
        ("phrase", "Økologisk mælk", "Maks. 6; pant; kun medlemmer"),
        ("name", "Økologisk letmælk", ""),
        ("description", "Mælk", "Økologisk"),
        ("missing", "Mælk", "konventionel"),
        ("tie-b", "Økologisk letmælk", ""),
    ] {
        let mut row = base.clone();
        row["source_id"] = json!(id);
        row["heading"] = json!(heading);
        row["description"] = json!(description);
        row["membership"] = json!("required");
        row["quantity"] = json!({"kind":"exact","amount":"0.33","unit":"l"});
        row["quantity_basis"] = json!("per_piece");
        row["pieces"] = json!({"kind":"exact","count":6});
        row["store"] = json!({"store_id":"42","name":"Butik","location":null});
        rows.push(row);
    }
    manifest["offer_count"] = json!(rows.len());
    manifest["coverage"] = json!("landsdækkende");
    fs::write(
        dir.0.join("tjek.jsonl"),
        format!(
            "{manifest}\n{}",
            rows.iter().map(|r| format!("{r}\n")).collect::<String>()
        ),
    )
    .unwrap();
    let full = success(dir.run(&["find", "ØKOLOGISK mælk", "--kilde", "tjek", "--limit", "20"]));
    let results = full["data"]["results"].as_array().unwrap();
    let ids: Vec<_> = results
        .iter()
        .map(|r| r["offer"]["source_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["phrase", "name", "tie-b", "description"]);
    let reversed =
        success(dir.run(&["find", "mælk økologisk", "--kilde", "tjek", "--limit", "20"]));
    let mut reversed_ids: Vec<_> = reversed["data"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["offer"]["source_id"].as_str().unwrap())
        .collect();
    reversed_ids.sort();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(reversed_ids, sorted);
    let mut pages = Vec::new();
    for offset in ["0", "2"] {
        let page = success(dir.run(&[
            "find",
            "ØKOLOGISK mælk",
            "--kilde",
            "tjek",
            "--limit",
            "2",
            "--offset",
            offset,
            "--compact",
        ]));
        assert_eq!(page["data"]["total"], 4);
        assert_eq!(page["data"]["has_more"], offset == "0");
        assert_eq!(page["data"]["sources"][0]["coverage"], "landsdækkende");
        pages.extend(page["data"]["results"].as_array().unwrap().iter().cloned());
    }
    for (compact, full) in pages.iter().zip(results) {
        for field in [
            "source",
            "source_id",
            "chain_id",
            "store",
            "heading",
            "description",
            "price",
            "quantity",
            "quantity_basis",
            "pieces",
            "validity",
            "membership",
        ] {
            assert_eq!(compact[field], full["offer"][field], "{field}");
        }
        assert_eq!(compact["unit_price"], full["unit_price"]);
        assert_eq!(compact["validity_status"], full["validity_status"]);
    }
    let empty = success(dir.run(&["find", "mælk", "--offset", "999"]));
    assert_eq!(empty["data"]["results"].as_array().unwrap().len(), 0);
    assert_eq!(empty["data"]["has_more"], false);
    let human = dir.run(&[
        "find",
        "ØKOLOGISK mælk",
        "--kilde",
        "tjek",
        "--limit",
        "1",
        "--offset",
        "3",
        "--human",
    ]);
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.contains("Mælk") && text.contains("offset 3") && text.contains("flere: nej"));
    assert!(!text.contains("Økologisk letmælk") && !text.trim_start().starts_with('{'));
    for args in [
        &["find", " "][..],
        &["find", "mælk", "--offset", "-1"],
        &["find", "mælk", "--offset", "x"],
        &["find", "mælk", "--compact", "--human"],
    ] {
        let out = dir.run(args);
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
}
