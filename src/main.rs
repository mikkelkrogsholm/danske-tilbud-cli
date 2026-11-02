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
/// Hent hele kildens datasæt og gem kun et fuldført snapshot.
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
    #[usage(long)]
    limit: Option<usize>,
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
                        json!({"schema_version":1,"command":"hent","error":"incomplete_run","source":id,"complete":false,"source_errors":result["source_errors"],"invalid_records":result["invalid_records"],"offer_count":result["offer_count"],"coverage":result["coverage"],"coverage_limitation":result["coverage_limitation"],"message":result["message"]}),
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
            let ids: Vec<&str> = match a.kilde.as_deref() {
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
            }
            let limit = a.limit.unwrap_or(20);
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
                    if !offer
                        .heading
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&query)
                        && !offer
                            .description
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&query)
                    {
                        return None;
                    }
                    let status = engine::validity_status(&offer.validity, now);
                    if status != ValidityStatus::Current
                        && !(a.include_unknown && status == ValidityStatus::Unknown)
                    {
                        return None;
                    }
                    let unit_price = engine::unit_price(&offer)
                        .map(|u| json!({"ore":u.ore.to_string(),"unit":u.unit}));
                    Some((offer, status, unit_price))
                })
                .collect::<Vec<_>>();
            offers.sort_by_cached_key(|(o, _, _)| {
                (
                    o.heading.as_deref().unwrap_or("").to_lowercase(),
                    o.chain_id.clone(),
                    o.source_id.clone(),
                    serde_json::to_string(o).unwrap(),
                )
            });
            let total = offers.len();
            let results = offers.into_iter().take(limit).map(|(offer,status,unit_price)| json!({"offer":offer,"validity_status":status,"unit_price":unit_price})).collect::<Vec<_>>();
            let value = envelope(
                "find",
                json!({"query":a.tekst,"source":a.kilde,"sources":sources,"complete":!failures,"chain":a.butik,"fetched_at":fetched_at,"total":total,"limit":limit,"results":results}),
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
    for adapter in adapters {
        results.push(refresh(adapter, dir).await);
    }
    let complete = results.iter().all(|r| r["status"] == "published");
    envelope("hent", json!({"complete":complete,"sources":results}))
}
async fn refresh(adapter: &impl SourceAdapter, dir: &Path) -> Value {
    let fetched_at = Utc::now().fixed_offset();
    let run = engine::run(adapter, &FetchOptions::default()).await;
    let (status, message) = if !run.complete {
        ("incomplete", None)
    } else {
        match storage::publish(dir, &run, fetched_at) {
            Ok(()) => ("published", None),
            Err(e) => ("failed", Some(e.to_string())),
        }
    };
    json!({"source":run.source.id,"status":status,"complete":status == "published","message":message,"fetched_at":fetched_at,"offer_count":run.offers.len(),"promotion_count":run.offers.iter().filter(|o| o.offer.price_kind == PriceKind::Offer).count(),"catalog_count":run.offers.iter().filter(|o| o.offer.price_kind == PriceKind::Catalog).count(),"duplicates":run.duplicates,"source_errors":run.source_errors,"invalid_records":run.invalid_records.iter().map(|v| &v.reason).collect::<Vec<_>>(),"coverage":run.coverage,"coverage_limitation":run.source.coverage_limitation})
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
                if human && value["command"] == "hent" {
                    println!("{value}");
                    human::write(&value, true).expect("kunne ikke skrive resultat");
                } else if human {
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
