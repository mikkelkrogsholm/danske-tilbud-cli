# Tilbudskontrakt, version 1

Et JSON-objekt repræsenterer én kildepost. `schema_version` er `1`, og
`source` + `source_id` identificerer posten hos kilden. `chain_id` identificerer
kæden; `heading`, butik og lokation kan være `null`, når kilden ikke oplyser dem. Teksten i `description` bør bevare
kildens betingelser, f.eks. mængdebegrænsninger. `source_reference` er en
reference til den oprindelige post, og `fetched_at` er et tidspunkt med eksplicit
UTC-offset. EAN og produkt-ID er strenge, så foranstillede nuller bevares.

Alle priser er DKK i hele øre; `amount_ore: null` betyder ukendt pris og er
forskellig fra `0`. `before_ore` er en eventuel sammenligningspris
fra kilden. Den er ikke nødvendigvis en tidligere normalpris: betydning og
betingelser skal bevares i beskrivelsen, hvis kilden oplyser dem. Ingen
prisnormalisering gennem binære flydende kommatal indgår i kontrakten.
`price_kind` er `offer` eller `catalog`.

Mængder er valgfrie, enten `exact` eller `range`, med decimaltal som strenge og
enhed som kildens tekst, f.eks. `l`, `kg` eller `stk`. `quantity_basis` er
`per_piece` når mængden gælder hver enkelt del, `total` når den gælder hele
pakken, og `null` når dette er ukendt eller mængden mangler. `pieces` angiver
valgfrit antal dele som `exact` med `count`, eller `range` med `min` og `max`;
`null` betyder ukendt eller ikke oplyst antal. Dermed kan f.eks. 6 × 330 ml
repræsenteres uden at miste hverken antal eller størrelse pr. del. Periodegrænser kan
uafhængigt være `instant` med offset, `date` uden tidspunkt eller `unknown`.
Der antages ikke et tidspunkt eller en tidszone for en dato. `membership` er
`required`, `not_required` eller `unknown`; ukendt er ikke det samme som nej.
Null betyder manglende oplysninger for de valgfrie felter. Modellen behandler
ikke et bestemt tidspunkt som inklusivt eller eksklusivt, og validerer
ikke kilde-ID, intervallers rækkefølge eller schema-version ved deserialisering.

Eksempel med offset og præcis decimalmængde:

```json
{
  "schema_version": 1,
  "source": "eksempel",
  "source_id": "123",
  "chain_id": "kaede-1",
  "store": {"store_id": "42", "name": "Butik", "location": "København"},
  "heading": "Sodavand",
  "description": "Maks. 6 stk.; derefter 20,53 kr. pr. stk.",
  "price": {"amount_ore": 1000, "before_ore": 2053},
  "quantity": {"kind": "exact", "amount": "1.5", "unit": "l"},
  "quantity_basis": "per_piece",
  "pieces": null,
  "validity": {"starts": {"precision": "instant", "value": "2026-11-02T08:00:00+01:00"}, "ends": {"precision": "unknown"}},
  "membership": "unknown",
  "fetched_at": "2026-11-02T09:00:00+01:00",
  "source_reference": "https://example.com/offers/123",
  "ean": "0123456789012",
  "product_id": null,
  "price_kind": "offer"
}
```

Eksempel med datopræcision, interval og katalogpris:

```json
{
  "schema_version": 1,
  "source": "eksempel",
  "source_id": "124",
  "chain_id": "kaede-1",
  "store": null,
  "heading": "Æbler",
  "description": null,
  "price": {"amount_ore": 2500, "before_ore": null},
  "quantity": {"kind": "range", "min": "4", "max": "6", "unit": "stk"},
  "quantity_basis": "total",
  "pieces": null,
  "validity": {"starts": {"precision": "date", "value": "2026-11-02"}, "ends": {"precision": "date", "value": "2026-11-08"}},
  "membership": "not_required",
  "fetched_at": "2026-11-02T08:00:00Z",
  "source_reference": "https://example.com/products/124",
  "ean": null,
  "product_id": "produkt-124",
  "price_kind": "catalog"
}
```

## Kilder og normalisering

En kildeadapter beskriver metadata og filtreringsmuligheder og modtager
`FetchOptions`. Den henter og mapper kildens svar til `RawOffer` uden at
omregne priser eller enheder. `SourceBatch` kan indeholde både poster,
dækningsbeskrivelse og kildefejl, så et delvist resultat ikke ligner et
fuldstændigt resultat. Engine returnerer gyldige tilbud, afviste råposter med
årsager, kildefejl og antal dubletter. `SourceBatch.complete` er som standard
false og skal bekræftes af kilden; motorens `RunResult.complete` kræver desuden,
at der ikke er kildefejl eller ugyldige poster. Tjek-adapterens kildedækning er
beskrevet i [Tjek-kilden](tjek.md).

Engine accepterer DKK som eksakte decimalstrenge og kræver hele, ikke-negative
øre. Decimalafvigelser på højst 0,00000001 øre fra nærmeste hele øre
rundes til dette øretal for at håndtere observeret kildestøj; større
afvigelser afvises. Den normaliserer gram til kg, ml og cl til liter samt
kendte styk-aliaser, herunder Tjeks `pcs`, til `stk`; ukendte enheder og valutaer afvises. Enhedsprisen angives i decimal
øre pr. kg, liter eller stk. og beregnes kun ved kendt pris og præcis, positiv
samlet mængde. Ved `per_piece` kræves et præcist antal dele. Intervaller og
ukendt basis giver ingen enhedspris. Decimaldivision kan afrundes til
`Decimal`-typens præcision.

Datoer og tidspunkter valideres hver for sig; rækkefølgen kontrolleres kun når
begge grænser har samme præcision. En blanding af dato og tidspunkt vurderes
ikke kronologisk, da datoen ikke har en tidszone. Tidspunkter indlæses som
RFC 3339; kildeformater som `+0000` skal mappes af den konkrete adapter.
Ved gyldighedskontrol er datoers grænser inklusive i Europe/Copenhagen. Præcis
lighed med et grænsetidspunkt giver ukendt gyldighed, medmindre den anden
grænses oplysninger allerede viser fremtidig eller udløbet gyldighed.
Dubletidentitet består af kilde, kilde-ID, kæde, hele butiksobjektet og
periodegrænserne. Ved samme identitet beholdes den leksikografisk første
serialiserede normaliserede post, uafhængigt af indlæsningsrækkefølge; andre
butikker, perioder og kilder sammenlægges ikke.
