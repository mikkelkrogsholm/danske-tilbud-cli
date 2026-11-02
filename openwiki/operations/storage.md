---
type: operations
title: Snapshot-lagring
description: Lokal JSONL-lagring, komplethed, låsning, atomisk offentliggørelse og valg af datamappe.
tags: [storage, snapshots, jsonl]
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T14:59:18.920Z
sources:
  - id: openwiki-source-62eef0b8799186dd9e2faae9
    resource: repo://src/storage.rs
generated: { by: "codex", at: "2026-11-02T14:59:18.920Z" }
---

# Snapshot-lagring

`storage::publish` gemmer ét aktuelt snapshot pr. kilde; `storage::read` læser det. CLI'ets `hent` offentliggør en fuldført kildekørsel, og `find` søger lokalt i snapshotet. Et resultat skal udtrykkeligt være komplet og uden kilde- eller valideringsfejl; ellers bevares det seneste snapshot. Komplethed gælder kildens angivne scope: Tjeks Bilka-katalogvej udelader fritstående direkte tilbud efter 1000.

## Format og validering

`<source>.jsonl` begynder med et manifest (`record_type: "snapshot"`, `schema_version: 1`, kilde, `fetched_at`, `written_at`, dækning, `complete` og `offer_count`). Derefter følger ét kanonisk `Offer` pr. linje, ikke rå svar eller beregnede enhedspriser. Et komplet tomt snapshot består kun af manifestet med antal 0. Hver linje skal afsluttes med newline. Læseren afviser tomme, beskadigede, afbrudte, inkomplette eller ukendte formater, kildemismatch og uoverensstemmende antal; tilbuddene valideres igen.

## Lås, skrivning og fejl

Læsning og skrivning tager samme ikke-blokerende fillås `<source>.lock`. En samtidig adgang, der ikke kan få låsen, giver fejl; låsefilen kan blive liggende. Kilde-id må kun indeholde ASCII-bogstaver, cifre, bindestreg og understregning.

Skrivning opretter en unik ejerskabsmærket midlertidig fil i samme mappe, skriver og flusher data, synkroniserer filen og genlæser den til validering før atomisk rename. Ved almindelig fejl ryddes den egenoprettede midlertidige fil, og det tidligere snapshot bevares ved fejl før rename. Nedbrud kan efterlade en midlertidig fil. Der oprettes ingen historik eller backups.

## Datamappe

Et eksplicit `--data-dir` efter kommandoen tilsidesætter `TILBUD_DATA_DIR`, som igen tilsidesætter standarden. På macOS bruges `~/Library/Application Support/danske-tilbud-cli`; på Linux bruges `$XDG_DATA_HOME/danske-tilbud-cli` eller `~/.local/share/danske-tilbud-cli`. Se [kom godt i gang](../quickstart.md) for kommandoer og [dataforløbet](../architecture/data-pipeline.md) for komplethed og normalisering.
