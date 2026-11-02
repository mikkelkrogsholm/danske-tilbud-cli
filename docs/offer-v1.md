# Tilbudskontrakt, version 1

Et JSON-objekt repræsenterer én kildepost. `schema_version` er `1`, og
`source` + `source_id` identificerer posten hos kilden. `chain_id` identificerer
kæden; butik og lokation kan være `null`. Teksten i `description` bør bevare
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
ikke endnu et bestemt tidspunkt som inklusivt eller eksklusivt, og validerer
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
