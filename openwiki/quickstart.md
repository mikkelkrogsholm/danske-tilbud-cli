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
  - id: openwiki-source-ccfe8cc4504f25c304787a46
    resource: repo://src/engine.rs
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
generated: { by: "codex", at: "2026-11-02T14:58:10.972Z" }
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T14:58:10.972Z
---

# Kom godt i gang

Tilbud henter Tjek-tilbud til et lokalt snapshot og søger derefter uden netværkskald. Byg og prøv fra projektets rod (Rust 1.91 eller nyere):

```sh
cargo build
cargo run -- --help
cargo run -- kilder
cargo run -- beskriv tjek
cargo run -- hent --kilde tjek
cargo run -- find kaffe --butik netto --limit 20
cargo run -- find mælk --include-unknown --data-dir /sti/til/data
```

`kilder` og `beskriv` viser metadata og eventuelt seneste snapshot; `hent` erstatter kun et snapshot efter en komplet kørsel. `find` søger uden hensyn til store og små bogstaver i navn og beskrivelse og udelader katalogpriser. `--butik` vælger et kæde-id; `--kilde` vælger kilde og standarden er `tjek`. `total` er match før `--limit`, som er 20 som standard. Resultaterne indeholder original gyldighed, status og beregnet enhedspris, når den kan beregnes.

Som standard vises kun beviseligt aktuelle tilbud. `--include-unknown` medtager også ukendt gyldighed, men ikke poster der vides at være fremtidige eller udløbne. Datogrænser vurderes efter Europe/Copenhagen med inklusiv slutdato; tidsinstanter sammenlignes præcist.

Succesfulde kommandoer skriver et JSON-objekt med `schema_version`, `command` og `data` til stdout. Fejl giver ikke-nul exitkode og diagnostik på stderr. `--data-dir` står efter kommandoen og tilsidesætter `TILBUD_DATA_DIR` og platformens standard. Se [snapshot-lagring](operations/storage.md) for placering, format og fejlhåndtering. Tjek-dækningen er begrænset som beskrevet på [kildesiden](sources/tjek.md).

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
