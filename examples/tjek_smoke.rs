use tilbud::{
    engine,
    source::FetchOptions,
    tjek::{CHAINS, TjekAdapter},
};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut adapter = TjekAdapter::new();
    if let Some(chain) = std::env::args().nth(1) {
        if !CHAINS.iter().any(|(id, _)| *id == chain) {
            panic!("unknown chain");
        }
        adapter = adapter.select_chains(&[&chain]).unwrap();
    }
    let result = engine::run(&adapter, &FetchOptions::default()).await;
    println!(
        "coverage={:?}; offers={}; duplicates={}; invalid={}; errors={:?}",
        result.coverage,
        result.offers.len(),
        result.duplicates,
        result.invalid_records.len(),
        result.source_errors
    );
    let mut reasons = std::collections::BTreeMap::new();
    for invalid in &result.invalid_records {
        *reasons
            .entry((&invalid.record.chain_id, &invalid.reason))
            .or_insert(0usize) += 1;
    }
    println!("invalid reasons by chain={reasons:?}");
    if !result.source_errors.is_empty() {
        std::process::exit(1);
    }
}
