# Graph Report - tilbud  (2026-11-02)

## Corpus Check
- 20 files · ~15,598 whitespace-separated tokens
- Verdict: corpus is large enough that graph structure adds value.

## Integritet og analyse
- Kodeudtræk: 20 filer, 398 noder, 965 rå relationer. Den simple analyse har 962 relationer; tre par har parallelle relationstyper. Eksporten bevarer alle 965 rå relationer.
- AST-producentens 12 heuristiske suppressionssteder er ikke dokumentation for et tabsløst udtræk. Ingen semantisk dokumentudtræk eller API-kald.

## Summary
- 398 nodes · 962 edges · 15 communities (12 shown, 3 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 9 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Normalisering og gyldighed
- CLI kommandoer og søgning
- Snapshot og fillåsning
- Kildeadaptere og integrationstest
- Tjek mapping og kæder
- Motortest og batch
- BilkaToGo adapter
- CLI integrationstest
- REMA tilbud og kampagner
- Tjek adaptertest
- Læsbar præsentation
- Søgebenchmark
- Isoleret testwrapper
- Tilbud pakke

## God Nodes (most connected - your core abstractions)
1. `SourceMetadata` - 23 edges
2. `RawOffer` - 23 edges
3. `SourceBatch` - 19 edges
4. `Offer` - 18 edges
5. `SourceAdapter` - 17 edges
6. `FetchOptions` - 16 edges
7. `TjekAdapter` - 16 edges
8. `BilkaToGoAdapter` - 15 edges
9. `normalize_batch()` - 14 edges
10. `Membership` - 14 edges

## Surprising Connections (you probably didn't know these)
- `adapter()` --references--> `BilkaToGoAdapter`  [EXTRACTED]
  tests/bilkatogo.rs → src/bilkatogo.rs
- `dedup_is_order_independent_and_keeps_store_and_period_distinct()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `fractional_ore_noise_has_a_small_exact_tolerance()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `invalid_records_are_visible_and_do_not_suppress_valid_records()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs
- `major_and_minor_price_units_convert_without_float_or_fractional_ore()` --calls--> `normalize_batch()`  [INFERRED]
  tests/engine.rs → src/engine.rs

## Import Cycles
- None detected.

## Communities (15 total, 3 thin omitted)

### Community 0 - "Normalisering og gyldighed"
Cohesion: 0.06
Nodes (50): boundary(), calculate_unit_price(), exact_product(), identity(), InvalidRecord, money(), normalize(), normalize_quantity() (+42 more)

### Community 1 - "CLI kommandoer og søgning"
Cohesion: 0.06
Nodes (36): run(), Beskriv, Command, Beskriv, Find, Hent, Kilder, describe() (+28 more)

### Community 2 - "Snapshot og fillåsning"
Cohesion: 0.08
Nodes (27): RunResult, validate_offer(), concurrent_process_writer_is_rejected(), data_dir(), incomplete_and_injected_failure_preserve_bytes(), invalid(), lock(), lock_child() (+19 more)

### Community 3 - "Kildeadaptere og integrationstest"
Cohesion: 0.12
Nodes (21): adapter(), config(), missing_config_and_invalid_selected_price_fail_without_snapshot(), missing_store_never_uses_root_or_other_store_and_ambiguous_quantity_is_retained(), page(), pagination_scope_and_price_are_preserved(), refresh_once_on_rejection_and_stop_on_second_rejection(), rejects_drift_duplicates_truncation_and_malformed_data() (+13 more)

### Community 4 - "Tjek mapping og kæder"
Cohesion: 0.13
Nodes (10): boundary(), CHAINS, LIMIT, map_offer(), MAX_PAGES, nullable_string(), number(), parse_time() (+2 more)

### Community 5 - "Motortest og batch"
Cohesion: 0.15
Nodes (12): normalize_batch(), batch(), block_on(), dedup_is_order_independent_and_keeps_store_and_period_distinct(), fractional_ore_noise_has_a_small_exact_tolerance(), invalid_records_are_visible_and_do_not_suppress_valid_records(), major_and_minor_price_units_convert_without_float_or_fractional_ore(), metadata() (+4 more)

### Community 6 - "BilkaToGo adapter"
Cohesion: 0.14
Nodes (12): amount(), BilkaToGoAdapter, config_value(), map_product(), MAX_PAGES, PAGE_SIZE, RESPONSE_LIMIT, ROOT_LIMIT (+4 more)

### Community 7 - "CLI integrationstest"
Cohesion: 0.26
Nodes (11): agent_search_ranks_pages_and_preserves_compact_terms(), combined_search_preserves_sources_and_reports_partial_failure(), human_narrow_search_keeps_long_names_conditions_and_unknown_prices(), human_uses_same_filtered_results_and_is_plain_when_piped(), local_search_filters_and_reports_snapshot(), missing_and_corrupt_snapshots_fail_usefully(), NEXT, offline_fetch_pipeline_publishes_files_consumed_by_real_cli() (+3 more)

### Community 8 - "REMA tilbud og kampagner"
Cohesion: 0.16
Nodes (7): exact_boundary_and_unparseable_price_are_not_invented(), LIMIT, map_product(), MAX_PAGES, newest_campaign_wins_and_equal_decimal_values_are_not_conflicts(), number(), text()

### Community 9 - "Tjek adaptertest"
Cohesion: 0.33
Nodes (16): adapter(), bad_row_is_reported_but_following_offer_is_kept(), capped_prefix(), catalog_pages_are_traversed_before_offer_pages(), catalog_store_is_preserved_and_dates_filter_in_copenhagen(), empty_catalogs_are_not_claimed_as_offer_coverage(), filtering_store_date_and_country(), fixture_server() (+8 more)

### Community 10 - "Læsbar præsentation"
Cohesion: 0.26
Nodes (13): age(), boundary(), coverage(), description(), document(), fetch_uses_the_supplied_source_limitation(), limitation(), money() (+5 more)

### Community 12 - "Isoleret testwrapper"
Cohesion: 0.40
Nodes (3): test.sh script, TILBUD_TEST_TMPDIR, TMPDIR

## Knowledge Gaps
- **33 isolated node(s):** `Current`, `Expired`, `Future`, `Unknown`, `NotRequired` (+28 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 110 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SourceMetadata` connect `CLI kommandoer og søgning` to `Normalisering og gyldighed`, `Snapshot og fillåsning`, `Tjek mapping og kæder`, `Motortest og batch`, `BilkaToGo adapter`, `REMA tilbud og kampagner`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **What connects `Current`, `Expired`, `Future` to the rest of the system?**
  _33 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Normalisering og gyldighed` be split into smaller, more focused modules?**
  _Cohesion score 0.058384547848990345 - nodes in this community are weakly interconnected._
- **Why does `Membership` connect `Normalisering og gyldighed` to `Kildeadaptere og integrationstest`, `Tjek mapping og kæder`, `Motortest og batch`, `BilkaToGo adapter`, `REMA tilbud og kampagner`, `Tjek adaptertest`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Should `CLI kommandoer og søgning` be split into smaller, more focused modules?**
  _Cohesion score 0.061507936507936505 - nodes in this community are weakly interconnected._
- **Why does `PriceKind` connect `Normalisering og gyldighed` to `CLI kommandoer og søgning`, `Kildeadaptere og integrationstest`, `Tjek mapping og kæder`, `Motortest og batch`, `BilkaToGo adapter`, `REMA tilbud og kampagner`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Should `Snapshot og fillåsning` be split into smaller, more focused modules?**
  _Cohesion score 0.08055152394775036 - nodes in this community are weakly interconnected._