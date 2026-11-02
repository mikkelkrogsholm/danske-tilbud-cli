---
type: integration
title: Tjek-kilden
description: Tjek-adapterens hentning, dækningsgrænser, filtre, fejl og dokumenterede liveobservationer.
tags: [tjek, source-adapter, pagination]
sources:
  - id: openwiki-source-138c2e8e14eae17ee441a093
    resource: repo://docs/tjek.md
  - id: openwiki-source-ccfe8cc4504f25c304787a46
    resource: repo://src/engine.rs
  - id: openwiki-source-b55a21a31ede1b56cd31a6a6
    resource: repo://src/main.rs
  - id: openwiki-source-dc57eb67cb7ae0022f82d3af
    resource: repo://src/tjek.rs
generated: { by: "codex", at: "2026-11-02T16:08:33.221Z" }
verified:
  - by: openwiki/0.7.3
    at: 2026-11-02T16:08:33.221Z
---

# Tjek-kilden

`TjekAdapter` er en biblioteksadapter for `https://squid-api.tjek.com/v2`; CLI-kommandoen `hent --kilde tjek` bruger den til at hente og gemme et komplet snapshot. `new()` vælger alle 14 kendte kæder. `select_chains` vælger et udsnit og afviser en tom liste eller ukendte kæde-id'er. `with_base` og en timeout kan bruges til offline fixtures. Kædeudsnit samt butiks- og datofilter ved hentning er biblioteksvalg; CLI henter hele Tjek-kilden. Adapteren leverer `SourceBatch` til [motoren](../architecture/data-pipeline.md), hvor poster normaliseres og ugyldige poster rapporteres.

## Hentning og dækning

For hver valgt kæde hentes `/offers` med dealer-id, limit 100 og offset, sekventielt indtil en side er kortere end limit. Listen er begrænset til 200 sider; gentagne eller for store sider og andre sidefejl giver synlige fejl og delvis dækning. Et tomt direkte feed giver ukendt tilbudsdækning og en kildefejl.

Ved `PAGINATION_MAX_REACHED` skiftes til `/catalogs` for kæden og derefter `/offers` pr. returneret katalog med dealer- og katalog-id. Dækningen gælder de hentede kataloger; fritstående direkte tilbud efter grænsen på 1000 er ikke omfattet. Tom katalogliste giver ukendt dækning og fejl. Katalog- eller sidefejl rapporteres, mens tilgængelige poster bevares. Dækningsbeskrivelsen angiver vej, antal og om resultatet er delvist eller komplet inden for sit scope.

Adapteren kontrollerer dealer og dansk land på hver post, samt katalogtilhørsforhold og eventuelt butiks-id i katalogvejen. Butiksfilteret medtager landsdækkende tilbud og tilbud for den valgte butik. Datofilteret anvendes på hentede poster som overlap med kalenderdagen i Europe/Copenhagen; det er ikke en historisk søgning. Ukendte eller ugyldige gyldighedstider ved datofilter giver fejl for posten.

## HTTP og mapping

Standardtimeout er 10 sekunder og højst tre forsøg. Netværksfejl og 5xx genforsøges med kort stigende ventetid. Ved 429 kræves en gyldig `Retry-After`; forsinkelser over fem sekunder returnerer fejl frem for et for tidligt genforsøg. Mangelfulde svar og afvigende dealer, land, katalog eller butiksrelation bliver kildefejl frem for tavs succes.

JSON-priser læses uden binære flydende kommatal. Kildens `pre_price` er en sammenligningspris, ikke nødvendigvis en tidligere normalpris. Størrelse og antal bevares separat, og `pieces.max` behandles ikke som pakkeantal. Medlemskab er ukendt, da kilden ikke giver en verificeret struktureret markering. Beskrivelsen bevares, men øvrige vilkår kan ikke altid udledes struktureret.

Ved vellykket hentning fjernes beviseligt udløbne poster før snapshotet publiceres. [Søgeguiden](../workflows/search.md) beskriver kilde- og kædefiltre, rangering og pagination.

## Dokumenteret liveobservation

En fuld hybridkørsel den 2. november 2026 ca. 14:44 UTC gav ifølge [kildedokumentationen](../../docs/tjek.md) 5.568 normaliserede tilbud, ingen dubletter, afvisninger eller kildefejl. Bilka brugte katalogvejen med ovenstående dækningsgrænse; tre føtex-poster uden overskrift blev bevaret. En separat stikprøve samme dag omfattede 14 kæder. Observationerne er tidsbundne, ikke garantier for aktuel eller fuld dækning.

Kør `cargo run --example tjek_smoke` for en ny livekørsel eller tilføj `-- rema1000` for én kæde. Se [REMA-kilden](rema.md) og [BilkaToGo](bilkatogo.md) for andre kilder og [kom godt i gang](../quickstart.md) for almindelig build og test; [offline tests](../../tests/tjek.rs) dækker adapterens grænser og fejlveje.
