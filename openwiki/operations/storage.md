---
type: operations
title: Snapshot-lagring
description: Lokal JSONL-lagring, komplethed, låsning, atomisk offentliggørelse og valg af datamappe.
tags: [storage, snapshots, jsonl]
sources:
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-2161525089da862e880baf7a
    resource: repo://scripts/test.sh
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
  - id: openwiki-source-62eef0b8799186dd9e2faae9
    resource: repo://src/storage.rs
generated: { by: "codex", at: "2026-11-02T15:43:42.669Z" }
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T15:45:43.206Z
---

# Snapshot-lagring

`storage::publish` gemmer ét aktuelt snapshot pr. kilde; `storage::read` læser det. CLI'ets `hent` offentliggør en fuldført kildekørsel, og `find` søger lokalt i snapshotet. `hent --alle` publicerer kilder uafhængigt: en fejlet kilde bevarer sit tidligere snapshot, mens fuldførte kilder kan offentliggøres. `find` springer manglende snapshots over; ulæselige snapshots giver delvist resultat og ikke-nul exitkode. Et resultat skal udtrykkeligt være komplet og uden kilde- eller valideringsfejl; ellers bevares det seneste snapshot. Komplethed gælder kildens angivne scope: Tjeks Bilka-katalogvej udelader fritstående direkte tilbud efter 1000.

## Format og validering

`<source>.jsonl` begynder med et manifest (`record_type: "snapshot"`, `schema_version: 1`, kilde, `fetched_at`, `written_at`, dækning, `complete` og `offer_count`). Derefter følger ét kanonisk `Offer` pr. linje, ikke rå svar eller beregnede enhedspriser. `offer_count` tæller alle gemte rækker, herunder katalogrækker; CLI’ets `promotion_count` og `catalog_count` viser fordelingen ved hentning. Et komplet tomt snapshot består kun af manifestet med antal 0. Hver linje skal afsluttes med newline. Læseren afviser tomme, beskadigede, afbrudte, inkomplette eller ukendte formater, kildemismatch og uoverensstemmende antal; tilbuddene valideres igen.

## Lås, skrivning og fejl

Læsning og skrivning tager samme ikke-blokerende fillås `<source>.lock`. En samtidig adgang, der ikke kan få låsen, giver fejl; låsefilen kan blive liggende. Kilde-id må kun indeholde ASCII-bogstaver, cifre, bindestreg og understregning.

Skrivning opretter en unik ejerskabsmærket midlertidig fil i samme mappe, skriver og flusher data, synkroniserer filen og genlæser den til validering før atomisk rename. Ved almindelig fejl ryddes den egenoprettede midlertidige fil, og det tidligere snapshot bevares ved fejl før rename. Nedbrud kan efterlade en midlertidig fil. Der oprettes ingen historik eller backups.

## Datamappe

Et eksplicit `--data-dir` efter kommandoen tilsidesætter `TILBUD_DATA_DIR`, som igen tilsidesætter standarden. På macOS bruges `~/Library/Application Support/danske-tilbud-cli`; på Linux bruges `$XDG_DATA_HOME/danske-tilbud-cli` eller `~/.local/share/danske-tilbud-cli`. Se [kom godt i gang](../quickstart.md) for kommandoer og [dataforløbet](../architecture/data-pipeline.md) for komplethed og normalisering.

## Udvikling og disk

Projektets `./scripts/test.sh` isolerer build og testdata i en egen midlertidig mappe og rydder den ved normal afslutning og håndterede signaler. SIGKILL og nedbrud kan efterlade data. Dev- og testprofiler deaktiverer debug-info og incremental compilation. Vælg Worktrunk-miljøet i Codex; projektet har ikke tilmeldt cachekloning. Se [README](../../README.md) og [kom godt i gang](../quickstart.md).
