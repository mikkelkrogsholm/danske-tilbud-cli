---
type: architecture
title: Tilbudsdata og normalisering
description: Kildekontrakten og motorens behandling af tilbud, fejl, identitet og enhedspriser.
tags: [data-pipeline, normalization, offers]
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T16:08:33.221Z
sources:
  - id: openwiki-source-23775c3de52f3ab95a13cb8b
    resource: repo://README.md
  - id: openwiki-source-ccfe8cc4504f25c304787a46
    resource: repo://src/engine.rs
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
  - id: openwiki-source-44a34a357da21d258ec6b5c7
    resource: repo://src/offer.rs
  - id: openwiki-source-836b4d70d3eac35c620705c6
    resource: repo://src/rema.rs
  - id: openwiki-source-4cab5a0ac2324da50f386806
    resource: repo://src/source.rs
  - id: openwiki-source-62eef0b8799186dd9e2faae9
    resource: repo://src/storage.rs
generated: { by: "codex", at: "2026-11-02T16:08:33.221Z" }
---

# Tilbudsdata og normalisering

Biblioteket eksponerer en kildekontrakt og en generisk motor. `SourceAdapter` beskriver metadata og en asynkron `fetch` med valgfrie butiks- og datofiltre. Adapteren har ansvaret for transport, paginering og oversættelse af svar til kildenære `RawOffer`-poster; kontrakten angiver også, hvilke filtre kilden understøtter. [Tjek-adapteren](../sources/tjek.md) henter data over HTTP som biblioteksfunktion; CLI-kommandoen `hent` bruger adapteren, mens `find` søger i lagrede snapshots. Kildens metadata ejer kæde-id’er, kapabiliteter og dækningsbegrænsninger. CLI’et samler [Tjek](../sources/tjek.md), [REMA](../sources/rema.md) og [BilkaToGo](../sources/bilkatogo.md) i en fælles enum, som implementerer adapterkontrakten. Prisvalg sker i den enkelte kilde før den fælles normalisering.

`fetch` returnerer en `SourceBatch` med poster, valgfri dækningsbeskrivelse og kildefejl. `engine::run` henter batchen og sender den sammen med metadata til `normalize_batch`. Resultatet `RunResult` indeholder normaliserede tilbud, ugyldige poster med årsag, kildefejl, dækning og et antal dubletter. En delvis kildefejl forhindrer således ikke, at gyldige poster i samme batch behandles. Batchen skal udtrykkeligt angive `complete`; `RunResult.complete` er kun sand, når batchen er komplet og hverken kilde- eller valideringsfejl forekommer.

## Validering og præcision

Motoren accepterer kun skemaversion 1 og DKK; kildeidentitet, postens kilde-id, kæde-id og kildereference skal være udfyldt. Overskrift er valgfri; tom eller blank overskrift bliver ukendt. Priser leveres som decimalstrenge med eksplicit kildeenhed: Tjek og REMA bruger Major (kroner), BilkaToGo Minor (øre). Motoren omregner til ikke-negative hele øre med kontrolleret decimalaritmetik. Ugyldige og negative beløb samt overflow afvises. For Major afrundes beløb inden for 0,00000001 øre fra nærmeste hele øre; større brøkdele afvises. Minor kræver præcist hele øre. `Offer` gemmer beløbet som heltalsøre; mængder bruger decimaler, ikke binære flydende kommatal.

Datoer bevares som datoer uden antaget klokkeslæt eller tidszone; tidsinstanter og hentetidspunkt skal være RFC 3339. Motoren afviser omvendt gyldighed, når begge grænser har samme præcision. Blandede præcisioner sammenlignes ikke. Den fælles gyldighedsvurdering bruger Copenhagen-datoer med inklusiv slutdato; præcis lighed med en tidspunktgrænse giver ukendt, medmindre den anden grænse afgør fremtid eller udløb.

Mængder skal være positive og intervaller ordnede. Gram omregnes til kg, centiliter og milliliter til liter, mens styk og `pcs` normaliseres til `stk`. Ukendte enheder og unøjagtig eller umulig omregning afvises. Antal styk skal være positivt; et mængdegrundlag uden mængde er ugyldigt.

Enhedspris beregnes kun, når pris, præcis mængde og mængdegrundlag er kendt. For mængde pr. styk kræves også et præcist antal styk; intervaller giver ingen enhedspris. Resultatet er decimal-øre pr. kg, liter eller styk, og division kan afrundes til decimaltypens præcision. `NormalizedOffer` samler `Offer` og den valgfrie enhedspris.

## Identitet og stabilitet

Dubletidentiteten består af kilde, kilde-id, kæde-id, hele butiksobjektet og gyldighed. Struktureret serialisering undgår kollisioner fra skilletegn. For samme identitet beholdes det leksikografisk første serialiserede `Offer`; antal dubletter tælles, og resultater samt ugyldige poster ordnes deterministisk uanset inputrækkefølge. EAN og produkt-id indgår ikke i identiteten. Tilbud fra forskellige kilder flettes ikke sammen, selv ved samme navn eller EAN; CLI samler kildevis resultater, og præsentationen kan vælges som JSON eller human.

`storage::publish` gemmer kun komplette resultater uden kilde- eller valideringsfejl. Efter indlæsning genbruges motorens `validity_status` og `unit_price` til at klassificere gyldighed og beregne enhedspris. Se [snapshot-lagring](../operations/storage.md) for format og fejlhåndtering. Se [kom godt i gang](../quickstart.md) for aktuel brug og test. Kontrakten og eksemplerne findes i [offer-v1](../../docs/offer-v1.md); [motortestene](../../tests/engine.rs) dækker blandt andet normalisering, fejl og dubletter.

## Rensning og søgning

Efter fuld validering fjerner `engine::remove_expired` kun tilbud, som er beviseligt udløbne ved hentningens fælles referencetid. Ufuldstændige kørsler ændres ikke af rensningen og kan ikke publiceres; en vellykket hentning erstatter kildens snapshot atomisk. Se [lagring](../operations/storage.md).

CLI’et læser de enkelte snapshots og filtrerer og rangerer resultater i `find`; det er en præsentationsgrænse oven på den fælles datamodel, ikke en sammenfletning af kilder. [Søgeguiden](../workflows/search.md) beskriver ordmatch og output.
