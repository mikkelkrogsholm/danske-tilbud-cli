---
type: architecture
title: Tilbudsdata og normalisering
description: Kildekontrakten og motorens behandling af tilbud, fejl, identitet og enhedspriser.
tags: [data-pipeline, normalization, offers]
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T14:46:29.006Z
sources:
  - id: openwiki-source-ccfe8cc4504f25c304787a46
    resource: repo://src/engine.rs
  - id: openwiki-source-44a34a357da21d258ec6b5c7
    resource: repo://src/offer.rs
  - id: openwiki-source-4cab5a0ac2324da50f386806
    resource: repo://src/source.rs
generated: { by: "codex", at: "2026-11-02T14:46:29.006Z" }
---

# Tilbudsdata og normalisering

Biblioteket eksponerer en kildekontrakt og en generisk motor. `SourceAdapter` beskriver metadata og en asynkron `fetch` med valgfrie butiks- og datofiltre. Adapteren har ansvaret for transport, paginering og oversættelse af svar til kildenære `RawOffer`-poster; kontrakten angiver også, hvilke filtre kilden understøtter. [Tjek-adapteren](../sources/tjek.md) henter data over HTTP som biblioteksfunktion; den er endnu ikke tilsluttet CLI-søgning.

`fetch` returnerer en `SourceBatch` med poster, valgfri dækningsbeskrivelse og kildefejl. `engine::run` henter batchen og sender den sammen med metadata til `normalize_batch`. Resultatet `RunResult` indeholder normaliserede tilbud, ugyldige poster med årsag, kildefejl, dækning og et antal dubletter. En delvis kildefejl forhindrer således ikke, at gyldige poster i samme batch behandles.

## Validering og præcision

Motoren accepterer kun skemaversion 1 og DKK; kildeidentitet, postens kilde-id, kæde-id og kildereference skal være udfyldt. Overskrift er valgfri; tom eller blank overskrift bliver ukendt. Priser leveres som decimalstrenge og omregnes til ikke-negative hele øre med kontrolleret decimalaritmetik. Ugyldige og negative beløb samt overflow afvises. Beløb inden for 0,00000001 øre fra nærmeste hele øre afrundes dertil; større brøkdele afvises. `Offer` gemmer beløbet som heltalsøre; mængder bruger decimaler, ikke binære flydende kommatal.

Datoer bevares som datoer uden antaget klokkeslæt eller tidszone; tidsinstanter og hentetidspunkt skal være RFC 3339. Motoren afviser omvendt gyldighed, når begge grænser har samme præcision. Blandede præcisioner sammenlignes ikke.

Mængder skal være positive og intervaller ordnede. Gram omregnes til kg, centiliter og milliliter til liter, mens styk og `pcs` normaliseres til `stk`. Ukendte enheder og unøjagtig eller umulig omregning afvises. Antal styk skal være positivt; et mængdegrundlag uden mængde er ugyldigt.

Enhedspris beregnes kun, når pris, præcis mængde og mængdegrundlag er kendt. For mængde pr. styk kræves også et præcist antal styk; intervaller giver ingen enhedspris. Resultatet er decimal-øre pr. kg, liter eller styk, og division kan afrundes til decimaltypens præcision. `NormalizedOffer` samler `Offer` og den valgfrie enhedspris.

## Identitet og stabilitet

Dubletidentiteten består af kilde, kilde-id, kæde-id, hele butiksobjektet og gyldighed. Struktureret serialisering undgår kollisioner fra skilletegn. For samme identitet beholdes det leksikografisk første serialiserede `Offer`; antal dubletter tælles, og resultater samt ugyldige poster ordnes deterministisk uanset inputrækkefølge. EAN og produkt-id indgår ikke i identiteten.

Motoren gemmer ikke data og er ikke tilsluttet CLI'ets tilbudssøgning. Se [kom godt i gang](../quickstart.md) for aktuel brug og test. Kontrakten og eksemplerne findes i [offer-v1](../../docs/offer-v1.md); [motortestene](../../tests/engine.rs) dækker blandt andet normalisering, fejl og dubletter.
