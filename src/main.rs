mod human;

use chrono::Utc;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tilbud::{
    bilkatogo::BilkaToGoAdapter,
    engine::{self, ValidityStatus},
    offer::PriceKind,
    rema::RemaAdapter,
    source::{FetchOptions, SourceAdapter, SourceBatch, SourceMetadata},
    storage,
    tjek::TjekAdapter,
};
use usage_rs::{Args, Cli, Subcommands};

/// Søg i lokalt hentede danske dagligvaretilbud (JSON).
#[derive(Cli)]
#[usage(bin = "tilbud", version = "0.1.0")]
struct Tilbud {
    #[usage(subcommand)]
    command: Command,
}
#[derive(Subcommands)]
enum Command {
    Kilder(Kilder),
    Beskriv(Beskriv),
    Hent(Hent),
    Find(Find),
}
/// Vis tilgængelige kilder.
#[derive(Args)]
struct Kilder {
    /// Vis resultatet i et læsbart format.
    #[usage(long)]
    human: bool,
    /// Datamappe (ellers TILBUD_DATA_DIR eller platformens standard).
    #[usage(long)]
    data_dir: Option<PathBuf>,
}
/// Beskriv en kilde.
#[derive(Args)]
struct Beskriv {
    /// Vis resultatet i et læsbart format.
    #[usage(long)]
    human: bool,
    kilde: String,
    #[usage(long)]
    data_dir: Option<PathBuf>,
}
/// Hent hele kildens datasæt; udløbne poster fjernes ved næste vellykkede hentning.
#[derive(Args)]
struct Hent {
    /// Vis resultatet i et læsbart format.
    #[usage(long)]
    human: bool,
    #[usage(long)]
    kilde: Option<String>,
    /// Hent alle registrerede kilder.
    #[usage(long)]
    alle: bool,
    #[usage(long)]
    data_dir: Option<PathBuf>,
}
/// Søg lokalt i aktuelle tilbud.
#[derive(Args)]
struct Find {
    /// Vis resultatet i et læsbart format.
    #[usage(long)]
    human: bool,
    tekst: String,
    #[usage(long)]
    butik: Option<String>,
    #[usage(long)]
    kilde: Option<String>,
    /// Maksimalt antal resultater (standard: 20).
    #[usage(long)]
    limit: Option<usize>,
    /// Spring dette antal match over (standard: 0).
    #[usage(long)]
    offset: Option<usize>,
    /// Vis kompakt JSON til agenter; kan ikke kombineres med --human.
    #[usage(long)]
    compact: bool,
    #[usage(long)]
    data_dir: Option<PathBuf>,
    /// Medtag tilbud med ukendt gyldighed, når de ikke vides at være udløbet eller fremtidige.
    #[usage(long)]
    include_unknown: bool,
}
enum Source {
    Tjek(TjekAdapter),
    Rema(RemaAdapter),
    BilkaToGo(BilkaToGoAdapter),
}
impl SourceAdapter for Source {
    fn metadata(&self) -> &SourceMetadata {
        match self {
            Self::Tjek(s) => s.metadata(),
            Self::Rema(s) => s.metadata(),
            Self::BilkaToGo(s) => s.metadata(),
        }
    }
    async fn fetch(&self, options: &FetchOptions) -> SourceBatch {
        match self {
            Self::Tjek(s) => s.fetch(options).await,
            Self::Rema(s) => s.fetch(options).await,
            Self::BilkaToGo(s) => s.fetch(options).await,
        }
    }
}
const SOURCE_IDS: &[&str] = &["tjek", "rema", "bilkatogo"];

fn source(id: &str) -> Result<Source, String> {
    match id {
        "tjek" => Ok(Source::Tjek(TjekAdapter::new())),
        "rema" => Ok(Source::Rema(RemaAdapter::new())),
        "bilkatogo" => Ok(Source::BilkaToGo(BilkaToGoAdapter::new())),
        _ => Err(format!("Ukendt kilde: {id}")),
    }
}
fn dir(path: Option<PathBuf>) -> Result<PathBuf, String> {
    storage::data_dir(path.as_deref()).map_err(|e| e.to_string())
}
fn describe(adapter: &impl SourceAdapter, dir: &Path) -> Value {
    let m = adapter.metadata();
    let snapshot = if !dir.join(format!("{}.jsonl", m.id)).exists() {
        json!(null)
    } else {
        match storage::read(dir, &m.id) {
            Ok(s) => {
                json!({"fetched_at":s.manifest.fetched_at,"offer_count":s.manifest.offer_count,"promotion_count":s.offers.iter().filter(|o| o.price_kind == PriceKind::Offer).count(),"catalog_count":s.offers.iter().filter(|o| o.price_kind == PriceKind::Catalog).count(),"coverage":s.manifest.coverage})
            }
            Err(e) => json!({"error":e.to_string()}),
        }
    };
    json!({"id":m.id,"name":m.name,"capabilities":{"store_filter":m.capabilities.store_filter,"date_filter":m.capabilities.date_filter},"chains":m.chains,"coverage_limitation":m.coverage_limitation,"snapshot":snapshot})
}
fn envelope(command: &str, data: Value) -> Value {
    json!({"schema_version":1,"command":command,"data":data})
}
async fn execute(command: Command) -> Result<Value, Value> {
    match command {
        Command::Kilder(a) => {
            let d = dir(a.data_dir).map_err(error)?;
            Ok(envelope(
                "kilder",
                json!({"sources":[describe(&TjekAdapter::new(), &d), describe(&RemaAdapter::new(), &d), describe(&BilkaToGoAdapter::new(), &d)]}),
            ))
        }
        Command::Beskriv(a) => {
            let s = source(&a.kilde).map_err(error)?;
            let d = dir(a.data_dir).map_err(error)?;
            Ok(envelope("beskriv", describe(&s, &d)))
        }
        Command::Hent(a) => {
            if a.alle == a.kilde.is_some() {
                return Err(error("Vælg præcis én af --alle eller --kilde <id>".into()));
            }
            let d = dir(a.data_dir).map_err(error)?;
            if let Some(id) = a.kilde {
                let s = source(&id).map_err(error)?;
                let result = refresh(&s, &d).await;
                if result["status"] != "published" {
                    return Err(
                        json!({"schema_version":1,"command":"hent","error":"incomplete_run","source":id,"complete":false,"source_errors":result["source_errors"],"invalid_records":result["invalid_records"],"offer_count":result["offer_count"],"expired_removed":result["expired_removed"],"coverage":result["coverage"],"coverage_limitation":result["coverage_limitation"],"message":result["message"]}),
                    );
                }
                return Ok(envelope("hent", result));
            }
            let adapters: Vec<_> = SOURCE_IDS
                .iter()
                .map(|id| source(id).expect("registered source"))
                .collect();
            let value = refresh_all(&adapters, &d).await;
            let complete = value["data"]["complete"] == true;
            if complete {
                Ok(value)
            } else {
                Err(partial(
                    value,
                    "En eller flere kilder kunne ikke opdateres; se kildernes status.",
                ))
            }
        }
        Command::Find(a) => {
            let mut ids: Vec<&str> = match a.kilde.as_deref() {
                Some(id) => {
                    source(id).map_err(error)?;
                    vec![id]
                }
                None => SOURCE_IDS.to_vec(),
            };
            if let Some(chain) = &a.butik {
                if !ids
                    .iter()
                    .any(|id| source(id).unwrap().metadata().chains.contains(chain))
                {
                    return Err(error(format!("Ukendt butik: {chain}")));
                }
                ids.retain(|id| source(id).unwrap().metadata().chains.contains(chain));
            }
            if a.compact && a.human {
                return Err(error("--compact kan ikke kombineres med --human".into()));
            }
            let limit = a.limit.unwrap_or(20);
            let offset = a.offset.unwrap_or(0);
            if limit == 0 {
                return Err(error("Limit skal være større end 0".into()));
            }
            if a.tekst.trim().is_empty() {
                return Err(error("Søgetekst må ikke være tom".into()));
            }
            let d = dir(a.data_dir).map_err(error)?;
            let mut snapshots = Vec::new();
            let mut sources = Vec::new();
            let mut failures = false;
            for id in &ids {
                match storage::read(&d, id) {
                    Ok(snapshot) => {
                        sources.push(json!({"source":id,"status":"available","fetched_at":snapshot.manifest.fetched_at,"offer_count":snapshot.manifest.offer_count,"coverage":snapshot.manifest.coverage}));
                        snapshots.push(snapshot);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound && a.kilde.is_none() => {
                        sources.push(json!({"source":id,"status":"missing"}));
                    }
                    Err(e) => {
                        if a.kilde.is_some() {
                            return Err(error(format!(
                                "Kan ikke læse snapshot for {id}: {e}. Kør tilbud hent --kilde {id}"
                            )));
                        }
                        failures = true;
                        sources.push(json!({"source":id,"status":"failed","message":format!("Kan ikke læse snapshot for {id}: {e}. Kør tilbud hent --kilde {id}")}));
                    }
                }
            }
            if snapshots.is_empty() && !failures {
                return Err(error("Ingen snapshots tilgængelige. Kør tilbud hent --alle eller tilbud hent --kilde <id>".into()));
            }
            let fetched_at = if a.kilde.is_some() {
                snapshots.first().map(|s| s.manifest.fetched_at)
            } else {
                None
            };
            let query = a.tekst.to_lowercase();
            let words: Vec<_> = query.split_whitespace().collect();
            let phrase = words.join(" ");
            let now = Utc::now();
            let mut offers = snapshots
                .into_iter()
                .flat_map(|snapshot| snapshot.offers)
                .filter_map(|offer| {
                    if offer.price_kind != PriceKind::Offer
                        || a.butik.as_ref().is_some_and(|b| *b != offer.chain_id)
                    {
                        return None;
                    }
                    let heading = offer.heading.as_deref().unwrap_or("").to_lowercase();
                    let description = offer.description.as_deref().unwrap_or("").to_lowercase();
                    if !words
                        .iter()
                        .all(|word| heading.contains(word) || description.contains(word))
                    {
                        return None;
                    }
                    let rank = if heading.contains(&phrase) {
                        0
                    } else if words.iter().all(|word| heading.contains(word)) {
                        1
                    } else {
                        2
                    };
                    let status = engine::validity_status(&offer.validity, now);
                    if status != ValidityStatus::Current
                        && !(a.include_unknown && status == ValidityStatus::Unknown)
                    {
                        return None;
                    }
                    let unit_price = engine::unit_price(&offer)
                        .map(|u| json!({"ore":u.ore.to_string(),"unit":u.unit}));
                    Some((offer, status, unit_price, rank))
                })
                .collect::<Vec<_>>();
            offers.sort_by_cached_key(|(o, _, _, rank)| {
                (
                    *rank,
                    o.heading.as_deref().unwrap_or("").to_lowercase(),
                    o.chain_id.clone(),
                    o.source_id.clone(),
                    serde_json::to_string(o).unwrap(),
                )
            });
            let total = offers.len();
            let has_more = offset.saturating_add(limit) < total;
            let results = offers.into_iter().skip(offset).take(limit).map(|(offer,status,unit_price,_)| {
                if a.compact {
                    json!({"source":offer.source,"source_id":offer.source_id,"chain_id":offer.chain_id,"store":offer.store,"heading":offer.heading,"description":offer.description,"price":offer.price,"quantity":offer.quantity,"quantity_basis":offer.quantity_basis,"pieces":offer.pieces,"validity":offer.validity,"validity_status":status,"membership":offer.membership,"unit_price":unit_price})
                } else {
                    json!({"offer":offer,"validity_status":status,"unit_price":unit_price})
                }
            }).collect::<Vec<_>>();
            let value = envelope(
                "find",
                json!({"query":a.tekst,"source":a.kilde,"sources":sources,"complete":!failures,"chain":a.butik,"fetched_at":fetched_at,"total":total,"offset":offset,"limit":limit,"has_more":has_more,"results":results}),
            );
            if failures {
                Err(partial(
                    value,
                    "Mindst ét snapshot kunne ikke læses; resultatet er ufuldstændigt.",
                ))
            } else {
                Ok(value)
            }
        }
    }
}
async fn refresh_all<A: SourceAdapter>(adapters: &[A], dir: &Path) -> Value {
    let mut results = Vec::new();
    let now = Utc::now();
    for adapter in adapters {
        results.push(refresh_at(adapter, dir, now).await);
    }
    let complete = results.iter().all(|r| r["status"] == "published");
    envelope("hent", json!({"complete":complete,"sources":results}))
}
async fn refresh(adapter: &impl SourceAdapter, dir: &Path) -> Value {
    refresh_at(adapter, dir, Utc::now()).await
}
async fn refresh_at(adapter: &impl SourceAdapter, dir: &Path, now: chrono::DateTime<Utc>) -> Value {
    let fetched_at = now.fixed_offset();
    let mut run = engine::run(adapter, &FetchOptions::default()).await;
    let expired_removed = engine::remove_expired(&mut run, now);
    let (status, message) = if !run.complete {
        ("incomplete", None)
    } else {
        match storage::publish(dir, &run, fetched_at) {
            Ok(()) => ("published", None),
            Err(e) => ("failed", Some(e.to_string())),
        }
    };
    json!({"source":run.source.id,"status":status,"complete":status == "published","message":message,"fetched_at":fetched_at,"offer_count":run.offers.len(),"promotion_count":run.offers.iter().filter(|o| o.offer.price_kind == PriceKind::Offer).count(),"catalog_count":run.offers.iter().filter(|o| o.offer.price_kind == PriceKind::Catalog).count(),"duplicates":run.duplicates,"expired_removed":expired_removed,"source_errors":run.source_errors,"invalid_records":run.invalid_records.iter().map(|v| &v.reason).collect::<Vec<_>>(),"coverage":run.coverage,"coverage_limitation":run.source.coverage_limitation})
}
fn partial(mut value: Value, message: &str) -> Value {
    value["error"] = json!("partial_result");
    value["message"] = json!(message);
    value
}
fn error(message: String) -> Value {
    json!({"schema_version":1,"error":"request_failed","message":message})
}
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Tilbud::parse();
    let human = match &args.command {
        Command::Kilder(a) => a.human,
        Command::Beskriv(a) => a.human,
        Command::Hent(a) => a.human,
        Command::Find(a) => a.human,
    };
    match execute(args.command).await {
        Ok(value) => {
            if human {
                human::write(&value, false).expect("kunne ikke skrive resultat");
            } else {
                println!("{value}");
            }
        }
        Err(value) => {
            if value["error"] == "partial_result" {
                if human {
                    human::write(&value, false).expect("kunne ikke skrive resultat");
                } else {
                    println!("{value}");
                }
                eprintln!(
                    "{}",
                    value["message"]
                        .as_str()
                        .unwrap_or("Ufuldstændigt resultat")
                );
            } else if human {
                human::write(&value, true).expect("kunne ikke skrive fejl");
            } else {
                eprintln!("{value}");
            }
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod orchestration_tests {
    use super::*;
    use tilbud::source::SourceCapabilities;
    struct Fake {
        metadata: SourceMetadata,
        batch: SourceBatch,
    }
    impl SourceAdapter for Fake {
        fn metadata(&self) -> &SourceMetadata {
            &self.metadata
        }
        async fn fetch(&self, _: &FetchOptions) -> SourceBatch {
            self.batch.clone()
        }
    }
    fn fake(id: &str, complete: bool, coverage: &str) -> Fake {
        Fake {
            metadata: SourceMetadata {
                id: id.into(),
                name: id.into(),
                chains: vec![],
                coverage_limitation: "test".into(),
                capabilities: SourceCapabilities::default(),
            },
            batch: SourceBatch {
                complete,
                errors: if complete {
                    vec![]
                } else {
                    vec!["transport timeout".into()]
                },
                coverage: Some(coverage.into()),
                ..Default::default()
            },
        }
    }
    fn record(
        id: &str,
        starts: tilbud::source::RawBoundary,
        ends: tilbud::source::RawBoundary,
    ) -> tilbud::source::RawOffer {
        use tilbud::{
            offer::Membership,
            source::{PriceUnit, RawOffer},
        };
        RawOffer {
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
            starts,
            ends,
            membership: Membership::Unknown,
            fetched_at: "2026-01-01T00:00:00Z".into(),
            source_reference: "ref".into(),
            ean: None,
            product_id: None,
            price_kind: PriceKind::Offer,
        }
    }

    #[tokio::test]
    async fn refresh_filters_validated_batches_and_atomically_replaces_actual_snapshots() {
        use chrono::TimeZone;
        use tilbud::source::RawBoundary::{Date, Instant, Unknown};
        let base = std::env::var_os("TILBUD_TEST_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let dir = base.join(format!("tilbud-expiry-test-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 23, 30, 0).unwrap(); // Jan 2 in Copenhagen
        let mut source = fake("tjek", true, "test");
        source.batch.records = vec![
            record("past", Unknown, Date("2026-01-01".into())),
            record(
                "today",
                Date("2026-01-01".into()),
                Date("2026-01-02".into()),
            ),
            record(
                "future",
                Date("2026-01-03".into()),
                Date("2026-01-04".into()),
            ),
            record("unknown", Unknown, Unknown),
            record("equal", Unknown, Instant(now.to_rfc3339())),
            record(
                "instant-past",
                Unknown,
                Instant((now - chrono::Duration::seconds(1)).to_rfc3339()),
            ),
            record("discontinued", Unknown, Unknown),
        ];
        let other = fake("rema", true, "other");
        refresh_at(&other, &dir, now).await;
        let other_bytes = std::fs::read(dir.join("rema.jsonl")).unwrap();
        let first = refresh_at(&source, &dir, now).await;
        assert_eq!(first["expired_removed"], 2);
        assert_eq!(first["offer_count"], 5);
        let path = dir.join("tjek.jsonl");
        let bytes = std::fs::read_to_string(&path).unwrap();
        assert!(!bytes.contains("\"source_id\":\"past\""));
        assert!(!bytes.contains("instant-past"));
        assert!(bytes.contains("discontinued"));
        assert_eq!(storage::read(&dir, "tjek").unwrap().manifest.offer_count, 5);
        source
            .batch
            .records
            .retain(|r| r.source_id != "discontinued");
        let second = refresh_at(&source, &dir, now + chrono::Duration::days(1)).await;
        assert_eq!(second["expired_removed"], 4);
        let bytes = std::fs::read_to_string(&path).unwrap();
        assert!(!bytes.contains("discontinued"));
        assert!(!bytes.contains("\"source_id\":\"today\""));
        assert!(bytes.contains("future"));
        assert!(bytes.contains("unknown"));
        assert_eq!(storage::read(&dir, "tjek").unwrap().manifest.offer_count, 2);
        assert_eq!(std::fs::read(dir.join("rema.jsonl")).unwrap(), other_bytes);
        // A previous snapshot may itself contain expired offers: failures must
        // leave its exact bytes untouched, not opportunistically clean it.
        let old_source = Fake {
            metadata: source.metadata.clone(),
            batch: SourceBatch {
                complete: true,
                records: vec![record("old-expired", Unknown, Date("2026-01-01".into()))],
                ..Default::default()
            },
        };
        let old_run = engine::run(&old_source, &FetchOptions::default()).await;
        storage::publish(&dir, &old_run, now.fixed_offset()).unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(String::from_utf8_lossy(&before).contains("old-expired"));
        source.batch.complete = false;
        assert_eq!(refresh_at(&source, &dir, now).await["status"], "incomplete");
        assert_eq!(std::fs::read(&path).unwrap(), before);
        source.batch.complete = true;
        let mut invalid = record("invalid-expired", Unknown, Date("2026-01-01".into()));
        invalid.currency = "EUR".into();
        source.batch.records.push(invalid);
        let failed = refresh_at(&source, &dir, now).await;
        assert_eq!(failed["status"], "incomplete");
        assert_eq!(failed["expired_removed"], 0);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        source.batch.records = vec![record("expired", Unknown, Date("2026-01-01".into()))];
        let empty = refresh_at(&source, &dir, now).await;
        assert_eq!(empty["expired_removed"], 1);
        let snapshot = storage::read(&dir, "tjek").unwrap();
        assert_eq!(snapshot.manifest.offer_count, 0);
        assert!(snapshot.offers.is_empty());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 4);
        for name in ["tjek.jsonl", "tjek.lock", "rema.jsonl", "rema.lock"] {
            std::fs::remove_file(dir.join(name)).unwrap();
        }
        std::fs::remove_dir(dir).unwrap();
    }

    #[tokio::test]
    async fn independent_publish_preserves_timed_out_snapshot_and_replaces_history() {
        let base = std::env::var_os("TILBUD_TEST_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let dir = base.join(format!("tilbud-refresh-test-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let initial = [
            fake("tjek", true, "old"),
            fake("rema", true, "old"),
            fake("bilkatogo", true, "old"),
        ];
        assert_eq!(refresh_all(&initial, &dir).await["data"]["complete"], true);
        let old = std::fs::read(dir.join("rema.jsonl")).unwrap();
        let next = [
            fake("tjek", true, "new"),
            fake("rema", false, "timeout"),
            fake("bilkatogo", true, "new"),
        ];
        for _ in 0..2 {
            let result = refresh_all(&next, &dir).await;
            assert_eq!(result["data"]["complete"], false);
            assert_eq!(result["data"]["sources"][0]["status"], "published");
            assert_eq!(result["data"]["sources"][1]["status"], "incomplete");
            assert_eq!(result["data"]["sources"][2]["status"], "published");
            assert_eq!(std::fs::read(dir.join("rema.jsonl")).unwrap(), old);
            assert_eq!(
                storage::read(&dir, "tjek")
                    .unwrap()
                    .manifest
                    .coverage
                    .as_deref(),
                Some("new")
            );
            assert_eq!(
                storage::read(&dir, "bilkatogo")
                    .unwrap()
                    .manifest
                    .coverage
                    .as_deref(),
                Some("new")
            );
        }
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 6);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
