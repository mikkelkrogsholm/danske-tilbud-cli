//! Atomic, bounded JSONL snapshots. See `docs/storage.md`.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    engine::{RunResult, validate_offer},
    offer::Offer,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub record_type: String,
    pub schema_version: u32,
    pub source: String,
    pub fetched_at: DateTime<FixedOffset>,
    pub written_at: DateTime<FixedOffset>,
    pub coverage: Option<String>,
    pub complete: bool,
    pub offer_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub manifest: Manifest,
    pub offers: Vec<Offer>,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn source_name(source: &str) -> io::Result<()> {
    if source.is_empty()
        || !source
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(invalid(
            "source ID must contain only ASCII letters, digits, '-' or '_'",
        ));
    }
    Ok(())
}

/// Explicit override wins over TILBUD_DATA_DIR and platform defaults.
pub fn data_dir(override_dir: Option<&Path>) -> io::Result<PathBuf> {
    if let Some(p) = override_dir {
        return Ok(p.to_path_buf());
    }
    if let Some(p) = std::env::var_os("TILBUD_DATA_DIR").filter(|s| !s.is_empty()) {
        return Ok(p.into());
    }
    if !cfg!(target_os = "macos")
        && let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|s| !s.is_empty())
    {
        return Ok(PathBuf::from(xdg).join("danske-tilbud-cli"));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| invalid("HOME is unset; specify a data directory"))?;
    if cfg!(target_os = "macos") {
        Ok(PathBuf::from(home).join("Library/Application Support/danske-tilbud-cli"))
    } else {
        Ok(PathBuf::from(home).join(".local/share/danske-tilbud-cli"))
    }
}

fn lock(dir: &Path, source: &str) -> io::Result<File> {
    source_name(source)?;
    fs::create_dir_all(dir)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(format!("{source}.lock")))?;
    file.try_lock()
        .map_err(|e| io::Error::other(format!("source {source} is locked: {e}")))?;
    Ok(file)
}

fn read_unlocked(dir: &Path, source: &str) -> io::Result<Snapshot> {
    read_file(&dir.join(format!("{source}.jsonl")), source)
}
fn require_fields(value: &serde_json::Value, fields: &[&str]) -> io::Result<()> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid("expected JSON object"))?;
    for field in fields {
        if !object.contains_key(*field) {
            return Err(invalid(format!("missing required field {field}")));
        }
    }
    Ok(())
}
fn parse_file(file: File, source: &str) -> io::Result<Snapshot> {
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let read_line =
        |reader: &mut BufReader<File>, line: &mut String, number: usize| -> io::Result<bool> {
            line.clear();
            if reader.read_line(line)? == 0 {
                return Ok(false);
            }
            if !line.ends_with('\n') || line.trim().is_empty() {
                return Err(invalid(format!(
                    "line {number}: blank or truncated JSONL; fetch again"
                )));
            }
            Ok(true)
        };
    if !read_line(&mut reader, &mut line, 1)? {
        return Err(invalid("empty snapshot; fetch again"));
    }
    let manifest_value: serde_json::Value = serde_json::from_str(&line)
        .map_err(|e| invalid(format!("invalid manifest on line 1: {e}; fetch again")))?;
    require_fields(
        &manifest_value,
        &[
            "record_type",
            "schema_version",
            "source",
            "fetched_at",
            "written_at",
            "coverage",
            "complete",
            "offer_count",
        ],
    )
    .map_err(|e| invalid(format!("invalid manifest on line 1: {e}; fetch again")))?;
    let manifest: Manifest = serde_json::from_str(&line)
        .map_err(|e| invalid(format!("invalid manifest on line 1: {e}; fetch again")))?;
    if manifest.record_type != "snapshot" || manifest.schema_version != 1 || !manifest.complete {
        return Err(invalid(
            "unsupported or incomplete snapshot manifest; fetch again",
        ));
    }
    if manifest.source != source {
        return Err(invalid("manifest source mismatch; fetch again"));
    }
    let mut offers = Vec::new();
    let mut number = 2;
    while read_line(&mut reader, &mut line, number)? {
        let value: serde_json::Value = serde_json::from_str(&line)
            .map_err(|e| invalid(format!("invalid offer on line {number}: {e}; fetch again")))?;
        let check = || -> io::Result<()> {
            require_fields(
                &value,
                &[
                    "schema_version",
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
                    "fetched_at",
                    "source_reference",
                    "ean",
                    "product_id",
                    "price_kind",
                ],
            )?;
            require_fields(&value["price"], &["amount_ore", "before_ore"])?;
            if !value["store"].is_null() {
                require_fields(&value["store"], &["store_id", "name", "location"])?;
            }
            Ok(())
        };
        check()
            .map_err(|e| invalid(format!("invalid offer on line {number}: {e}; fetch again")))?;
        let offer: Offer = serde_json::from_str(&line)
            .map_err(|e| invalid(format!("invalid offer on line {number}: {e}; fetch again")))?;
        validate_offer(&offer)
            .map_err(|e| invalid(format!("invalid offer on line {number}: {e}; fetch again")))?;
        if offer.source != source {
            return Err(invalid(format!(
                "offer source mismatch on line {number}; fetch again"
            )));
        }
        offers.push(offer);
        number += 1;
    }
    if offers.len() != manifest.offer_count {
        return Err(invalid("offer count mismatch; fetch again"));
    }
    Ok(Snapshot { manifest, offers })
}

/// Readers and writers use the same nonblocking cross-process source lock.
pub fn read(dir: &Path, source: &str) -> io::Result<Snapshot> {
    let _lock = lock(dir, source)?;
    read_unlocked(dir, source)
}

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
struct OwnedTemp(Option<PathBuf>);
impl Drop for OwnedTemp {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            let _ = fs::remove_file(path);
        }
    }
}

/// Publish a complete run; `fetched_at` is the source run's fetch time, even for an empty run.
pub fn publish(dir: &Path, run: &RunResult, fetched_at: DateTime<FixedOffset>) -> io::Result<()> {
    publish_inner(dir, run, fetched_at, false)
}
fn publish_inner(
    dir: &Path,
    run: &RunResult,
    fetched_at: DateTime<FixedOffset>,
    fail_before_rename: bool,
) -> io::Result<()> {
    let source = &run.source.id;
    source_name(source)?;
    if !run.complete || !run.source_errors.is_empty() || !run.invalid_records.is_empty() {
        return Err(invalid("incomplete run; previous snapshot preserved"));
    }
    for normalized in &run.offers {
        validate_offer(&normalized.offer).map_err(invalid)?;
        if normalized.offer.source != *source {
            return Err(invalid("offer source mismatch"));
        }
    }
    let _lock = lock(dir, source)?;
    let manifest = Manifest {
        record_type: "snapshot".into(),
        schema_version: 1,
        source: source.clone(),
        fetched_at,
        written_at: Utc::now().fixed_offset(),
        coverage: run.coverage.clone(),
        complete: true,
        offer_count: run.offers.len(),
    };
    let (path, file) = loop {
        let path = dir.join(format!(
            ".{source}.{}.{}.tmp",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => break (path, file),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    };
    let mut temp = OwnedTemp(Some(path));
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &manifest)?;
    writer.write_all(b"\n")?;
    for normalized in &run.offers {
        serde_json::to_writer(&mut writer, &normalized.offer)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    drop(writer);
    // Validate the actual file bytes before publishing.
    let snapshot = read_file(temp.0.as_ref().unwrap(), source)?;
    if snapshot.manifest != manifest {
        return Err(invalid("written manifest mismatch"));
    }
    if fail_before_rename {
        return Err(io::Error::other("injected pre-rename failure"));
    }
    fs::rename(
        temp.0.as_ref().unwrap(),
        dir.join(format!("{source}.jsonl")),
    )?;
    temp.0 = None;
    Ok(())
}

fn read_file(path: &Path, source: &str) -> io::Result<Snapshot> {
    parse_file(File::open(path)?, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{NormalizedOffer, normalize_batch},
        source::{SourceBatch, SourceCapabilities, SourceMetadata},
    };
    use std::{
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    // A concurrently spawned child can inherit another test's locked file during
    // fork/exec. Keep subprocess creation separate from unrelated lock tests.
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let base = std::env::var_os("TILBUD_TEST_TMPDIR")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            loop {
                let p = base.join(format!(
                    "tilbud-storage-{}-{}",
                    std::process::id(),
                    NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&p) {
                    Ok(()) => return Self(p),
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(e) => panic!("{e}"),
                }
            }
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn time() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2026-11-02T09:00:00+01:00").unwrap()
    }
    fn run() -> RunResult {
        let source = SourceMetadata {
            chains: vec![],
            id: "eksempel".into(),
            name: "Eksempel".into(),
            coverage_limitation: "Eksempeldækning".into(),
            capabilities: SourceCapabilities::default(),
        };
        let mut result = normalize_batch(
            source,
            SourceBatch {
                complete: true,
                coverage: Some("all scopes".into()),
                ..Default::default()
            },
        );
        let example = include_str!("../docs/offer-v1.md")
            .split("```json\n")
            .nth(1)
            .unwrap()
            .split("\n```")
            .next()
            .unwrap();
        let mut offer: Offer = serde_json::from_str(example).unwrap();
        offer.heading = None;
        offer.pieces = Some(crate::offer::PieceCount::Exact { count: 6 });
        offer.product_id = Some("product".into());
        result.offers.push(NormalizedOffer {
            offer,
            unit_price: None,
        });
        result
    }
    #[test]
    fn roundtrip_and_repeated_writes_are_bounded() {
        let _serial = TEST_LOCK.lock().unwrap();
        let d = TestDir::new();
        let r = run();
        publish(&d.0, &r, time()).unwrap();
        let snapshot = read(&d.0, "eksempel").unwrap();
        assert_eq!(snapshot.offers, vec![r.offers[0].offer.clone()]);
        assert_eq!(snapshot.manifest.fetched_at, time());
        assert_eq!(snapshot.manifest.coverage.as_deref(), Some("all scopes"));
        publish(&d.0, &r, time()).unwrap();
        assert_eq!(read(&d.0, "eksempel").unwrap().offers.len(), 1);
        let mut names: Vec<_> = fs::read_dir(&d.0)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        names.sort();
        assert_eq!(names, ["eksempel.jsonl", "eksempel.lock"]);
    }
    #[test]
    fn unknown_prices_and_heading_survive_roundtrip() {
        let _serial = TEST_LOCK.lock().unwrap();
        let d = TestDir::new();
        let mut r = run();
        r.offers[0].offer.price.amount_ore = None;
        r.offers[0].offer.price.before_ore = None;
        assert!(r.offers[0].offer.heading.is_none());
        publish(&d.0, &r, time()).unwrap();
        assert_eq!(
            read(&d.0, "eksempel").unwrap().offers,
            vec![r.offers[0].offer.clone()]
        );
    }
    #[test]
    fn missing_corrupt_and_explicit_empty() {
        let _serial = TEST_LOCK.lock().unwrap();
        let d = TestDir::new();
        assert_eq!(
            read(&d.0, "eksempel").unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        let path = d.0.join("eksempel.jsonl");
        for bytes in ["", "\n", "{}\n", "{}", "{bad}\n"] {
            fs::write(&path, bytes).unwrap();
            assert!(read(&d.0, "eksempel").is_err(), "{bytes:?}");
        }
        let mut r = run();
        r.offers.clear();
        publish(&d.0, &r, time()).unwrap();
        assert!(read(&d.0, "eksempel").unwrap().offers.is_empty());
        let good = fs::read_to_string(&path).unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&good).unwrap();
        value["schema_version"] = 2.into();
        fs::write(&path, format!("{value}\n")).unwrap();
        assert!(
            read(&d.0, "eksempel")
                .unwrap_err()
                .to_string()
                .contains("unsupported")
        );
        value["schema_version"] = 1.into();
        value["source"] = "other".into();
        fs::write(&path, format!("{value}\n")).unwrap();
        assert!(
            read(&d.0, "eksempel")
                .unwrap_err()
                .to_string()
                .contains("source mismatch")
        );
        value["source"] = "eksempel".into();
        value["offer_count"] = 1.into();
        fs::write(&path, format!("{value}\n")).unwrap();
        assert!(
            read(&d.0, "eksempel")
                .unwrap_err()
                .to_string()
                .contains("count")
        );
        publish(&d.0, &run(), time()).unwrap();
        let good = fs::read_to_string(&path).unwrap();
        let (head, row) = good.split_once('\n').unwrap();
        let mut offer: serde_json::Value = serde_json::from_str(row).unwrap();
        offer["schema_version"] = 2.into();
        fs::write(&path, format!("{head}\n{offer}\n")).unwrap();
        assert!(
            read(&d.0, "eksempel")
                .unwrap_err()
                .to_string()
                .contains("schema version")
        );
        offer["schema_version"] = 1.into();
        let mut missing = offer.clone();
        missing.as_object_mut().unwrap().remove("heading");
        fs::write(&path, format!("{head}\n{missing}\n")).unwrap();
        assert!(
            read(&d.0, "eksempel")
                .unwrap_err()
                .to_string()
                .contains("missing required field heading")
        );
        offer["source"] = "other".into();
        fs::write(&path, format!("{head}\n{offer}\n")).unwrap();
        assert!(
            read(&d.0, "eksempel")
                .unwrap_err()
                .to_string()
                .contains("source mismatch")
        );
        fs::write(&path, good.trim_end()).unwrap();
        assert!(
            read(&d.0, "eksempel")
                .unwrap_err()
                .to_string()
                .contains("truncated")
        );
        assert!(read(&d.0, "../escape").is_err());
    }
    #[test]
    fn incomplete_and_injected_failure_preserve_bytes() {
        let _serial = TEST_LOCK.lock().unwrap();
        let d = TestDir::new();
        let mut r = run();
        r.offers[0].offer.validity.starts = crate::offer::TimeBoundary::Unknown;
        r.offers[0].offer.validity.ends =
            crate::offer::TimeBoundary::Date(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        publish(&d.0, &r, time()).unwrap();
        let path = d.0.join("eksempel.jsonl");
        let before = fs::read(&path).unwrap();
        r.complete = false;
        r.offers.clear();
        assert!(publish(&d.0, &r, time()).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        r.complete = true;
        let mut cleaned = run();
        cleaned.offers[0].offer.validity.starts = crate::offer::TimeBoundary::Unknown;
        cleaned.offers[0].offer.validity.ends =
            crate::offer::TimeBoundary::Date(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(
            crate::engine::remove_expired(&mut cleaned, time().to_utc()),
            1
        );
        assert!(publish_inner(&d.0, &cleaned, time(), true).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(read(&d.0, "eksempel").unwrap().offers.len(), 1);
        assert_eq!(fs::read_dir(&d.0).unwrap().count(), 2);
    }
    #[test]
    fn lock_child() {
        let Some(dir) = std::env::var_os("TILBUD_LOCK_CHILD") else {
            return;
        };
        let dir = PathBuf::from(dir);
        let _lock = lock(&dir, "eksempel").unwrap();
        fs::write(dir.join("ready"), "ready").unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }
    #[test]
    fn concurrent_process_writer_is_rejected() {
        let _serial = TEST_LOCK.lock().unwrap();
        let d = TestDir::new();
        let r = run();
        publish(&d.0, &r, time()).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "storage::tests::lock_child", "--nocapture"])
            .env("TILBUD_LOCK_CHILD", &d.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let started = Instant::now();
        while !d.0.join("ready").exists() && started.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(10));
        }
        let ready = d.0.join("ready").exists();
        let result = if ready {
            Some(publish(&d.0, &r, time()))
        } else {
            None
        };
        drop(child.stdin.take());
        let status = child.wait().unwrap();
        assert!(ready && status.success());
        assert!(result.unwrap().unwrap_err().to_string().contains("locked"));
        fs::remove_file(d.0.join("ready")).unwrap();
    }
}
