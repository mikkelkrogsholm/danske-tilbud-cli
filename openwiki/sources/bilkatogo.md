---
type: integration
title: BilkaToGo-kilden
description: Offentlig tilbudsavis for et valgt lager, prisers enhed og krav til en komplet hentning.
tags: [bilkatogo, source-adapter, pagination]
sources:
  - id: openwiki-source-0b3c9083504c39afdb61ad7c
    resource: repo://docs/verification.md
  - id: openwiki-source-37441fc6d42f7dc6f56c109e
    resource: repo://src/bilkatogo.rs
  - id: openwiki-source-62eef0b8799186dd9e2faae9
    resource: repo://src/storage.rs
generated: { by: "codex", at: "2026-11-02T15:43:42.669Z" }
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T15:45:43.206Z
---

# BilkaToGo-kilden

BilkaToGo-adapteren henter den aktuelle tilbudsavis fra webshoppens offentlige produktsøgning. Scope er varer markeret i tilbudsavisen, som er i sortiment og på lager i det valgte lager eller butiks-scope; standarden er 1653, BilkaToGos leveringslager. Markeringen er ikke i sig selv bevis for rabat, og kilden er ikke hele produktkataloget eller alle fysiske butikkers priser. Bibliotekets `FetchOptions.store_id` kan vælge et andet numerisk scope; CLI bruger standarden.

## Transport og komplethed

Ved hver hentning finder adapteren den offentlige søgekonfiguration i webshoppens scripts. Den gemmer hverken scripts eller konfiguration og prøver højst én genindlæsning ved 401/403. Integrationen er uofficiel og kan ændres af udbyderen.

Søgningen pagineres med op til 100 sider à 1000 varer. Den kræver et udtømmende antal, konsistente totaler, forventet sidestørrelse og entydige produkt-id'er. Manglende data, uoverensstemmelser, sidegrænse eller andre kildefejl giver en ufuldstændig kørsel, som ikke erstatter snapshotet. Se [dataforløbet](../architecture/data-pipeline.md) og [lagring](../operations/storage.md).

## Pris og gyldighed

Pris og eventuel førpris læses fra det valgte scopes `storeData`; nul i `beforePrice` betyder ukendt førpris. Kildens priser er øre og sendes som `Minor` til motoren, som kræver hele øre. Uklare indholdstekster og oplyste vilkår bevares som beskrivelse. Tilbudsperiode og medlemskab antages ikke. `tilbud find kaffe --kilde bilkatogo --include-unknown` medtager derfor poster med ukendt gyldighed; uden flaget kræves beviseligt aktuelle tilbud.

## Dateret observation

Ved verifikationen 2. november 2026 blev 2.959 poster hentet i scope 1653. Kaffesøgningen gav 0 som standard og 37 med `--include-unknown`; tallene er et øjebliksbillede, ikke faste forventninger. Se [verifikation](../../docs/verification.md), [kildebeskrivelsen](../../docs/bilkatogo.md) og [offline tests](../../tests/bilkatogo.rs). [Kom godt i gang](../quickstart.md) viser installation og kommandoer.
