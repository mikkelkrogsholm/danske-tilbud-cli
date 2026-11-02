mod human;

use chrono::Utc;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tilbud::{
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
    kilde: String,
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
}
impl SourceAdapter for Source {
    fn metadata(&self) -> &SourceMetadata {
        match self {
            Self::Tjek(s) => s.metadata(),
            Self::Rema(s) => s.metadata(),
        }
    }
    async fn fetch(&self, options: &FetchOptions) -> SourceBatch {
        match self {
            Self::Tjek(s) => s.fetch(options).await,
            Self::Rema(s) => s.fetch(options).await,
        }
    }
}
fn source(id: &str) -> Result<Source, String> {
    match id {
        "tjek" => Ok(Source::Tjek(TjekAdapter::new())),
        "rema" => Ok(Source::Rema(RemaAdapter::new())),
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
                json!({"sources":[describe(&TjekAdapter::new(), &d), describe(&RemaAdapter::new(), &d)]}),
            ))
        }
        Command::Beskriv(a) => {
            let s = source(&a.kilde).map_err(error)?;
            let d = dir(a.data_dir).map_err(error)?;
            Ok(envelope("beskriv", describe(&s, &d)))
        }
        Command::Hent(a) => {
            let s = source(&a.kilde).map_err(error)?;
            let d = dir(a.data_dir).map_err(error)?;
            let fetched_at = Utc::now().fixed_offset();
            let run = engine::run(&s, &FetchOptions::default()).await;
            if !run.complete {
                return Err(
                    json!({"schema_version":1,"command":"hent","error":"incomplete_run","source":a.kilde,"complete":false,"source_errors":run.source_errors,"invalid_records":run.invalid_records.iter().map(|v| &v.reason).collect::<Vec<_>>(),"offer_count":run.offers.len(),"coverage":run.coverage,"coverage_limitation":run.source.coverage_limitation}),
                );
            }
            storage::publish(&d, &run, fetched_at).map_err(|e| error(e.to_string()))?;
            Ok(envelope(
                "hent",
                json!({"source":a.kilde,"complete":true,"fetched_at":fetched_at,"offer_count":run.offers.len(),"promotion_count":run.offers.iter().filter(|o| o.offer.price_kind == PriceKind::Offer).count(),"catalog_count":run.offers.iter().filter(|o| o.offer.price_kind == PriceKind::Catalog).count(),"duplicates":run.duplicates,"coverage":run.coverage,"coverage_limitation":run.source.coverage_limitation}),
            ))
        }
        Command::Find(a) => {
            let id = a.kilde.as_deref().unwrap_or("tjek");
            let selected_source = source(id).map_err(error)?;
            if let Some(chain) = &a.butik {
                if !selected_source.metadata().chains.contains(chain) {
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
            let snapshot = storage::read(&d, id).map_err(|e| {
                error(format!(
                    "Kan ikke læse snapshot for {id}: {e}. Kør tilbud hent --kilde {id}"
                ))
            })?;
            let query = a.tekst.to_lowercase();
            let now = Utc::now();
            let mut offers = snapshot
                .offers
                .into_iter()
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
            Ok(envelope(
                "find",
                json!({"query":a.tekst,"source":id,"chain":a.butik,"fetched_at":snapshot.manifest.fetched_at,"total":total,"limit":limit,"results":results}),
            ))
        }
    }
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
            if human {
                human::write(&value, true).expect("kunne ikke skrive fejl");
            } else {
                eprintln!("{value}");
            }
            std::process::exit(1);
        }
    }
}
