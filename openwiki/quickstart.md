---
type: guide
title: Kom godt i gang
description: Sådan køres den tidlige CLI-grundversion, og sådan bygges og testes projektet med begrænset diskforbrug.
tags: [quickstart, cli, testing]
sources:
  - id: openwiki-source-4905fab56ecf9fa5e1ebbf3f
    resource: repo://.cargo/config.toml
  - id: openwiki-source-651d1fb6c9e49916a916ab51
    resource: repo://Cargo.toml
  - id: openwiki-source-2161525089da862e880baf7a
    resource: repo://scripts/test.sh
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
  - id: openwiki-source-dc57eb67cb7ae0022f82d3af
    resource: repo://src/tjek.rs
generated: { by: "codex", at: "2026-11-02T14:46:29.006Z" }
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T14:46:29.006Z
---

# Kom godt i gang

Tilbud er et dansk dagligvare-CLI i en tidlig grundversion. CLI'et viser hjælp og meddeler ellers, at tilbudssøgning endnu ikke er implementeret. CLI'et henter eller viser endnu ikke tilbud og har ingen lagring. Biblioteket har en tilbudskontrakt, et adapterinterface, en normaliseringsmotor og en Tjek-adapter, der henter tilbud over netværket. Adapteren er endnu ikke koblet til CLI'et.

Projektet kræver Rust 1.91 eller nyere. Fra projektets rod kan du bygge programmet og se hjælpen:

```sh
cargo build
cargo run -- --help
```

## Test

Brug projektets wrapper til almindelige testkørsler:

```sh
./scripts/test.sh
./scripts/test.sh --lib -- --nocapture
```

Wrapperen sender argumenter videre til `cargo test`, men reserverer `--target-dir`. Den opretter en særskilt midlertidig mappe til build og testdata og rydder den ved normal afslutning, fejl og håndterede HUP-, INT- og TERM-signaler. SIGKILL, systemnedbrud eller strømsvigt kan efterlade mappen. Sæt eventuelt `CARGO` til en absolut sti til Cargo. Tests bør placere midlertidige filer under `TILBUD_TEST_TMPDIR` eller bruge ejerskabsbaseret oprydning. Almindelig `cargo test` omgår wrapperens isolering og oprydning.

Udviklings- og testprofilerne slår debug-information og incremental compilation fra for at begrænse diskforbruget, også for build-afhængigheder.

## Find videre

Læs [tilbudsdata og normalisering](architecture/data-pipeline.md) for flow, validering og begrænsninger og [Tjek-kilden](sources/tjek.md) for adapterens hentning og dækning. [Tilbudskontrakten](../docs/offer-v1.md) beskriver JSON-formatet med eksempler; [README](../README.md) beskriver også Worktrunk-miljøet.
