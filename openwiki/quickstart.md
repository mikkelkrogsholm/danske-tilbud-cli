---
type: guide
title: Kom godt i gang
description: Installation og brug af tre lokale kilder samt test med begrænset diskforbrug.
tags: [quickstart, cli, testing]
sources:
  - id: openwiki-source-4905fab56ecf9fa5e1ebbf3f
    resource: repo://.cargo/config.toml
  - id: openwiki-source-651d1fb6c9e49916a916ab51
    resource: repo://Cargo.toml
  - id: openwiki-source-0b3c9083504c39afdb61ad7c
    resource: repo://docs/verification.md
  - id: openwiki-source-2161525089da862e880baf7a
    resource: repo://scripts/test.sh
  - id: openwiki-source-ccfe8cc4504f25c304787a46
    resource: repo://src/engine.rs
  - id: openwiki-source-35aa34ef351101676460f475
    resource: repo://src/human.rs
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
generated: { by: "codex", at: "2026-11-02T15:43:42.669Z" }
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T15:45:43.206Z
---

# Kom godt i gang

Tilbud henter Tjek, REMA og BilkaToGo til lokale snapshots og søger derefter uden netværkskald. Installér fra en lokal kopi med Rust og Cargo på PATH:

```sh
cd /sti/til/tilbud
cargo install --locked --path .
tilbud kilder
tilbud beskriv bilkatogo
tilbud hent --alle
tilbud find kaffe
tilbud find kaffe --kilde bilkatogo --include-unknown
tilbud kilder --human
```

Cargo installerer normalt binæren i `~/.cargo/bin`, som skal være på PATH. `--root /sti/til/installationsmappe` vælger en anden destination. `Cargo.toml` deklarerer minimum Rust 1.91, men denne version er ikke afprøvet; [verifikationen](../docs/verification.md) brugte 1.99.1. Den installerede CLI kræver ikke Cargo, Graphify eller OpenWiki ved kørsel.

`kilder` og `beskriv` viser metadata og eventuelt seneste snapshot; `hent` erstatter kun et snapshot efter en komplet kørsel. `find` søger uden hensyn til store og små bogstaver i navn og beskrivelse og udelader katalogpriser. `--butik` vælger et kæde-id; `--kilde` vælger kilde; som standard søger `find` i alle tilgængelige snapshots. `total` er match før `--limit`, som er 20 som standard. `offer_count` er antal rækker, også katalogposter; `promotion_count` og `catalog_count` viser fordelingen ved hentning. Resultaterne indeholder original gyldighed, status og beregnet enhedspris, når den kan beregnes.

Som standard vises kun beviseligt aktuelle tilbud. `--include-unknown` medtager også ukendt gyldighed, men ikke poster der vides at være fremtidige eller udløbne. Datogrænser vurderes efter Europe/Copenhagen med inklusiv slutdato; ved præcis lighed med en tidspunktgrænse er status ukendt, medmindre den anden grænse afgør fremtid eller udløb.

Som standard skriver succesfulde kommandoer et JSON-objekt med `schema_version`, `command` og `data` til stdout. Tilføj `--human` efter kommandoen for en læsbar visning af de samme resultater; fejl giver ikke-nul exitkode og diagnostik på stderr. `--data-dir` står efter kommandoen og tilsidesætter `TILBUD_DATA_DIR` og platformens standard. Se [snapshot-lagring](operations/storage.md) for placering, format og fejlhåndtering. `hent --alle` publicerer fuldførte kilder uafhængigt og rapporterer delvise fejl med ikke-nul exitkode. Manglende snapshots springes over ved `find`, mens ulæselige snapshots giver delresultat og ikke-nul exitkode. Dækningen er begrænset som beskrevet på siderne for [Tjek](sources/tjek.md), [REMA](sources/rema.md) og [BilkaToGo](sources/bilkatogo.md).

## Test

Brug projektets wrapper til almindelige testkørsler:

```sh
./scripts/test.sh
./scripts/test.sh --lib -- --nocapture
```

Wrapperen sender argumenter videre til `cargo test`, men reserverer `--target-dir`. Den opretter en særskilt midlertidig mappe til build og testdata og rydder den ved normal afslutning, fejl og håndterede HUP-, INT- og TERM-signaler. SIGKILL, systemnedbrud eller strømsvigt kan efterlade mappen. Sæt eventuelt `CARGO` til en absolut sti til Cargo. Tests bør placere midlertidige filer under `TILBUD_TEST_TMPDIR` eller bruge ejerskabsbaseret oprydning. Almindelig `cargo test` omgår wrapperens isolering og oprydning.

Udviklings- og testprofilerne slår debug-information og incremental compilation fra for at begrænse diskforbruget, også for build-afhængigheder.

## Find videre

Læs [tilbudsdata og normalisering](architecture/data-pipeline.md) for flow, validering og begrænsninger og kildesiderne for [Tjek](sources/tjek.md), [REMA](sources/rema.md) og [BilkaToGo](sources/bilkatogo.md) for hentning og dækning. [Tilbudskontrakten](../docs/offer-v1.md) beskriver JSON-formatet med eksempler; [README](../README.md) beskriver også Worktrunk-miljøet.
