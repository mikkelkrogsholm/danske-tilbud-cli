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
