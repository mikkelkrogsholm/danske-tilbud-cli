# Tilbud: udvikling og diskforbrug

Følg brugerens globale developer-team/issue-manager-arbejdsgang: MAIN styrer og
reviewer; én worker ad gangen implementerer i et isoleret arbejdsområde.

## Worktrunk

- Primært checkout: `/Users/mikkelfreltoftkrogsholm/Projekter/tilbud`.
- Brug det native Codex-app-flow til oprettelse, tilknytning og recoverable
  arkivering af worktrees. Erstat ikke appens livscyklus med `wt switch`,
  `wt merge`, `wt remove` eller `wt step prune`.
- Vælg **Worktrunk**-miljøet. Setup kører
  `/opt/homebrew/bin/wt hook post-start --foreground`.
- Cache-seeding er vurderet og fravalgt indtil videre: projektet er nyt, og
  lavt diskforbrug prioriteres. Ingen `.codex/worktrunk.json` eller cachekopier
  skal oprettes automatisk. Vurder igen ved dokumenterede dyre genbygninger.
- Ved senere autoriseret enrollment: brug `$worktrunk-setup` og den vedligeholdte
  CLI i `/Users/mikkelfreltoftkrogsholm/Maintainance/mac-m3/worktrunk-setup`.
  Den stabile helper er `bin/clone-cache` samme sted. Del aldrig en skrivbar
  build-cache mellem worktrees via symlinks eller hardlinks, og føj aldrig
  build-cache til `.worktreeinclude`.

## Builds og tests

- Bevar Cargo-profiler med debug-info og incremental compilation slået fra
  for udvikling og tests, medmindre en konkret opgave kræver noget andet.
- Brug projektets `./scripts/test.sh` til almindelige testkørsler, når Cargo-pakken
  er oprettet. Wrapperen isolerer og rydder sin egen test-build og testdata.
- Nye tests skal placere genererede filer under `TILBUD_TEST_TMPDIR` eller bruge
  midlertidige mapper med ejerskabsbaseret oprydning. Versionsstyrede fixtures
  er kildemateriale og skal bevares.
- Slet kun egne genererede midlertidige filer. Ryd ikke globale Cargo-caches,
  andre projekters data eller aktive builds. Ingen periodisk baggrundsoprydning.
- SIGKILL, strømsvigt og systemnedbrud kan efterlade midlertidige filer; lov ikke
  ubetinget oprydning. Undersøg ejerskab og aktive jobs før manuel oprydning.

## Graphify og OpenWiki

- Aktuelt mandat: brugeren har bedt om løbende opdatering af Graphify og OpenWiki
  under gennemførelsen af EPIC #1. Opdatér dem ved meningsfulde kodeleverancer;
  wiki vedligeholdes efter at den relevante kildekode er stabil. MAIN ejer plan
  og review; workers udfører filændringer og værktøjskørsler, uden underdelegation.

- Graphify er installeret på brugerniveau. Ved `/graphify` skal den installerede
  skill følges. Projektet har endnu ingen genereret graf.
- Brug en eksisterende `graphify-out/graph.json` til relevante spørgsmål om
  arkitektur og relationer; verificér vigtige fund i den aktuelle kildekode.
  Almindelige lokale kodeopslag kan bruge direkte søgning og læsning.
- Opret eller opdatér ikke en graf alene på grund af denne vejledning. Byg og
  opdatér kun inden for en konkret autoriseret opgave. Ingen automatiske
  Git-hooks, watcher eller baggrundsjob er installeret til grafgenerering.
- OpenWiki er installeret som Codex-integration på brugerniveau via
  `openwiki integrations install codex`. Integrationen omfatter skill og MCP;
  Codex skal genstartes for at indlæse den nye integration.
- Projektet har endnu ingen genereret OpenWiki. Ved initialisering eller
  opdatering skal den installerede OpenWiki-skill og dens MCP-livscyklus følges.
  Eksisterende wiki bruges ved relevante spørgsmål og kontrolleres mod kilder.
