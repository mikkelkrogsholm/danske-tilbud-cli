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
