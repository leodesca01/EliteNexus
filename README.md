# Elite Nexus

**One native, fast, free and open-source companion app for Elite Dangerous and Odyssey** — replacing the pile of tools commanders run side by side
(EDDiscovery, ED Market Connector, Odyssey Materials Helper, Observatory + BioInsights, Exploration Buddy,
EDCoPilot-style voice alerts, OD Elite Tracker) and the websites you alt-tab to (INARA, Spansh, EDSM, EDSY/Coriolis,
EDAstro, ED Tools, EDRefCard).

Pure Rust. No Python, no Node.js, no browser engine. One executable, GPU-rendered UI (egui/wgpu).

**[Download the latest release](https://github.com/uruskan/EliteNexus/releases/latest)**, unzip it anywhere and run
`EliteNexus.exe` (Windows, 64-bit), or unpack the Linux `tar.gz` and run `./EliteNexus` (64-bit; it finds the journals
in the game's Proton prefix, and Settings › About adds it to your app menu; see [Linux](docs/USER_GUIDE.md#linux)).
On a Mac, build `Elite Nexus.app` from the source with `tools/macos/build-app.sh --install` (it finds the journals in
NotProton's, CrossOver's or Whisky's prefix; see [macOS](docs/USER_GUIDE.md#macos)).
There is no installer: everything it keeps lives in a `nexus-data` folder next to the program. Website: **[uruskan.github.io/EliteNexus](https://uruskan.github.io/EliteNexus/)**.

![Elite Nexus, an Elite Dangerous companion app: dashboard with commander ranks, active ship, location and session stats](docs/screenshots/dashboard.png)

## Make the cockpit yours: HUD colours, cockpit lights, glare

![Elite Dangerous HUD colour editor: an Empire blue theme in the live cockpit preview, with hostile contacts still red and friendly ones still green](docs/screenshots/hud-colours.png)

* **HUD colours**: a HUD colour editor built on the game's own setting, with community themes (credited to their
  authors), a colour wheel, saturation and brightness, and a live cockpit preview that warns when friend and foe would
  look alike. Applying writes only the three colour lines of `GraphicsConfigurationOverride.xml` and keeps your
  original file.
* **Keep hostile red, friendly green and shield blue.** The game's setting moves every HUD colour together, so a blue
  theme turns hostile contacts blue too. Switch this on and your theme colours only the HUD's orange.
* **Cockpit lights.** The game has no setting for them: Elite Nexus dims the cockpit's own lamps and glow and nothing
  else, with in-game keys for on/off (F9), how much light stays (F10) and reach (F11).
* **Glare and exposure.** Odyssey's auto-exposure darkens everything outside when the dashboard is bright; tame the
  glare and lift the darks with the game's own tone-mapping values, starting from community presets.

HUD colours and glare use the game's own settings file. Keeping friend-and-foe colours and dimming the cockpit lights
need an optional graphics hook: [3Dmigoto](https://github.com/bo3b/3Dmigoto) (open source), downloaded from its
official release, checked against pinned checksums, and installed or removed in one click from the Cockpit lights
tab. No game file is changed and Elite Nexus never ships the game's code: its patterns adjust the light and HUD
shaders as the game loads them. The hook can't run alongside EDHM, and Frontier tolerates mods like this without having
approved them.

| | |
|---|---|
| ![Elite Dangerous cockpit lights dimmer: installed, with light kept, reach and the in-game keys](docs/screenshots/cockpit-lights.png) **Cockpit lights** — dim the cockpit's own lights | ![Elite Dangerous Odyssey glare and auto-exposure fix: Morbad's Odyssey tweak applied, with less glare and the standard ACES curve](docs/screenshots/glare.png) **Glare and exposure** — less glare, brighter darks |

## Screenshots

| | |
|---|---|
| ![Exobiology: live sampling radar with two samples, colony-range circles and "OK to sample"](docs/screenshots/exobiology.png) **Exobiology** — live sampling radar, colony range, species value | ![Exploration: every body in the system with value, mapped value and highlights](docs/screenshots/exploration.png) **Exploration** — every body, value now vs. mapped, highlights |
| ![Ask: the AI co-pilot answering an exobiology question](docs/screenshots/ask.png) **Ask** — the AI co-pilot | ![Market: stations paying far above the galactic average](docs/screenshots/market.png) **Market** — abnormal prices across the galaxy |
| ![Navigation: the plotted route with star classes, scoopable stars and jump distances](docs/screenshots/navigation.png) **Navigation** — route, fuel stars, next system to the clipboard | ![Engineering: blueprint wishlist and the materials shopping list](docs/screenshots/engineering.png) **Engineering** — wishlist and shopping list |
| ![Fleet: every ship with its full engineered loadout](docs/screenshots/fleet.png) **Fleet** — loadouts, engineering, one-click EDSY/Coriolis | ![Fleet carrier: tritium, finances, capacity and crew](docs/screenshots/carrier.png) **Fleet carrier** — tritium, finances, crew |
| ![Galaxy map: everywhere you've been, with your ship and carrier](docs/screenshots/galaxy-map.png) **Galaxy map** — your travels, ship and carrier | ![Statistics: credits over time, income by category, jumps per day](docs/screenshots/statistics.png) **Statistics** — credits, income, jumps |
| ![Logbook: every journal event, searchable](docs/screenshots/logbook.png) **Logbook** — every journal event, searchable | ![Bindings: active presets, conflicts and every bound action](docs/screenshots/bindings.png) **Bindings** — presets, conflicts, reference card |
| ![Accounts & sync: EDDN, EDSM, Inara and EDAstro, all opt-in](docs/screenshots/accounts.png) **Accounts & sync** — EDDN, EDSM, Inara, EDAstro | |

## Your game, your data

* Your journals are only ever read. Uploads to EDDN, EDSM, Inara and EDAstro are opt-in, send only live events (never
  your history) and respect each service's rate limits.
* Game settings change only when you click, and only the values you change; your original settings file is kept.
  Whatever Elite Nexus puts in the game folder is listed there in a manifest and comes out again in one click.
* API keys are stored encrypted with Windows DPAPI (Linux and macOS have no DPAPI: there they sit in a settings file only your
  user account can read, and the app says so).

## Why it's faster

| | Elite Nexus |
|---|---|
| Full journal history load | **~40–60 ms** for 16.5k events (parallel parse, ~260 MB/s, 600k events/s) — EDD takes minutes on big histories |
| Live latency | journal + Status.json polled every **100 ms** (no missed writes, no notification lag) |
| Market data | **direct EDDN firehose** (~550 msgs/min) into a local SQLite DB — prices the moment any commander docks, before websites ingest them |
| Market queries | best price within 50 ly in **~1 ms**, galaxy-wide best price 0.2 ms, trade routes in 8–22 ms on an 80k-station / 4.4M-price database |
| Startup to usable | about one second |

## What works now

* **Ask** — a built-in AI co-pilot: ask in plain words ("where can I make the most money in the next hour?") and it
  answers from your ship, cargo, journal, the live market database and Spansh, with a knowledge base compiled into the
  app. Tools it can use: price deals (stations paying far above the galactic average), sell-my-cargo, trade routes,
  commodity search, nearest services, mining hotspots, system info, a system's planets (with who has set foot on them),
  untouched biology, route and money-route planning (routes land on the Navigation page), exobiology here, journal
  search. Setup is one pasted key (Google Gemini by default; Claude or a
  local model optional).

* **Dashboard** — commander, ranks & progress, credits, legal state, ship (fuel/hull/cargo/jump range/rebuy), location
  & system politics, session stats, current-system exploration value, holdings, fleet carrier, colonisation, activity feed.
* **Exploration** — every body as you FSS it: type, distance, value now vs. mapped (MattG formula, Odyssey bonus, first
  discovery/mapping, efficiency), gravity/temp/atmosphere, bio/geo signals, and Observatory-style highlights (ELW/WW/AW,
  terraformable, landable atmosphere, first footfall, close/wide rings, shepherd & nested moons, fast rotation/orbit,
  jumponium…). **Find valuable bodies** plans a scan tour (Spansh's Road to Riches) and widens the search out of the
  bubble by itself; **History** shows every system you scanned and whether its data was sold, refused or lost; codex.
* **Exobiology** — live **sampling radar**: previous sample points around you (oriented to your heading) with colony-range
  circles and a big "OK to sample / too close" readout; bio bodies with DSS genera; unsold Vista Genomics data with
  first-logged bonus (5×). **Species predictions** from the body's atmosphere, gravity, temperature, volcanism and parent
  stars before you land, with expected value (checked against your own history: for 15 of the 16 species you sampled,
  the top prediction for that genus was the right one). **Find planets**, two ways: a **bonus hunt** for planets where
  a valuable species should grow and nobody has a record of sampling it (first-logged bonus, ×5), which explains what
  it found and what the data can't prove, or **sure value**: planets where the species are already on record.
* **Market** — a local galaxy database fed live by EDDN, your own journal and (optionally) Spansh's daily dump:
  find the best/nearest place to buy or sell any commodity, **sell my cargo** (best single stop for the whole hold),
  trade routes for your ship and capital, nearest material traders / brokers / Interstellar Factors / Vista Genomics,
  mining hotspots with best sell prices, **price alerts** that fire seconds after any commander's market update
  matches your rule, and the full price list of the station you're docked at.
* **Navigation** — in-game route with scoopable/neutron/white-dwarf flags, non-scoopable streak warning, and **next
  system auto-copied to the clipboard after every jump**. Spansh planners built in: neutron, exact galaxy plotter
  (your ship's real FSD), fleet carrier (tritium); scan tours from Exploration and Exobiology land here too.
* **Fleet** — all ships, where they are (incl. in-transit), full loadouts with engineering & modifier deltas; one-click
  **Open in EDSY / Coriolis** with the exact build.
* **Engineering** — blueprint planner with a wishlist (or "upgrade to G5" from your ship), shopping list, material-trader
  plan, engineers (unlocked first, by distance), Odyssey suits & weapons, synthesis, materials and ship locker.
* **Missions** — active missions with expiry, cargo/kill progress, and a **massacre-stack tracker** (kills count once per
  issuing faction; shows exactly how many kills remain).
* **Mining** — live sessions: tonnes/hour, prospector quality, motherlodes.
* **Fleet carrier** — tritium, finances, capacity, crew/services, orders, jump schedule & history.
* **Colonisation** — construction depots with remaining tonnage per commodity, what's already in your hold, and
  **where to buy** each commodity nearest the site with stock for a full hold.
* **BGS & Powerplay** — what you did for which faction since the tick.
* **Logbook / Statistics / Galaxy map** — activity, every journal event with full-text search, travel, docking, a credits
  ledger by category, credit history, jumps per day, in-game statistics, and a map of everywhere you've been.
* **Bindings** — your active presets, conflicts, missing essentials, printable reference card.
* **HUD & cockpit** — HUD colours that can keep hostile red, friendly green and shield blue, a cockpit-light dimmer,
  and glare and exposure: see [Make the cockpit yours](#make-the-cockpit-yours-hud-colours-cockpit-lights-glare).
* **Accounts & sync** — strictly compliant EDDN contribution, EDSM journal sync, Inara profile sync, EDAstro. All opt-in;
  only live events (never history); persistent retry queues that respect each service's rate limits.
* **Alerts & voice** — toasts plus speech (Windows speech; speech-dispatcher on Linux; `say` on macOS) for high-value bodies, likely
  high-value biology, sample spacing, motherlodes, non-scoopable stars when fuel is low, interdictions, hull critical,
  mission completion, carrier jumps…
* **Compact HUD** — one click turns the window into a small always-on-top panel for a second screen.

## Build & run

Download the latest `EliteNexus-…-windows-x64.zip` from the GitHub Releases page, unzip anywhere, run `EliteNexus.exe`.
On Linux: `EliteNexus-…-linux-x64.tar.gz`, unpack anywhere, run `./EliteNexus`.

To build it yourself (Windows, Linux or macOS, [Rust](https://rustup.rs) stable):

```powershell
cargo run -p nexus-app                 # debug
cargo build --release -p nexus-app     # target\release\EliteNexus.exe (target/release/EliteNexus on Linux)
```

On macOS (with Xcode's Command Line Tools), `tools/macos/build-app.sh` builds and packs `target/macos/Elite Nexus.app`
and a zip of it; `--install` also copies the app to Applications.

On the original dev machine the toolchain is not on PATH: run `. .\tools\env.ps1` first. Pushing a tag such as `v0.1.0`
makes GitHub Actions build the Windows exe and the Linux binary and publish a Release
(`.github/workflows/release.yml`), with that version's `CHANGELOG.md` section as the notes. Working on the code, as a person or an AI agent? Read `AGENTS.md` first.

Command-line options: `--page <name>`, `--tab <name>`, `--journal-dir <dir>`, `--mute`, on Linux
`--install-desktop` / `--uninstall-desktop` (the app-menu entry),
`--screenshot <png> [--shot-delay <s>] [--size <w>x<h>] [--demo]`. A screenshot run renders off-screen, saves the PNG
and exits; it never uploads. The images in `docs/screenshots/` were made this way, e.g.
`EliteNexus.exe --page market --tab abnormal --screenshot docs\screenshots\market.png --shot-delay 8 --mute`.
Data (settings, DB, logs) is kept in `nexus-data\` next to the executable.

Dev tools: `cargo run --release -p nexus-journal --example scan_journals` (parse stats),
`cargo run --release -p nexus-model --example summary` (everything the model knows),
`cargo run --release -p nexus-journal --example replay -- <journal> <dir> [speed]` (simulate a live session),
`cargo run --release -p nexus-model --example predictions` (exobiology predictions vs. what you sampled),
`cargo run --release -p nexus-uplink --example validate` (every upload your journals would produce, checked against the
EDDN schemas — sends nothing), `cargo run --release -p nexus-galaxy --example seed_journal -- <db>` (galaxy DB from
your journals), `cargo run -p nexus-eddn --example live -- <db> <seconds>` (watch the live EDDN feed).

## Accounts you may want

* **EDSM** API key: https://www.edsm.net/en/settings/api
* **Inara** API key: https://inara.cz/elite/cmdr-settings-api/ (Inara only accepts whitelisted app names; it approved
  "Elite Nexus" on 2026-10-13, so your personal key is all sync needs. "Developer mode" stays on by default for the
  first weeks; see docs/research/inara.md)
* **Frontier Companion API (CAPI)** needs a client ID approved by Frontier (docs/research/capi.md) — optional; the
  journal already provides almost everything.

Keys are stored encrypted with Windows DPAPI (on Linux and macOS: unencrypted, readable only by your user account).

## Docs

* `AGENTS.md` — how to work on this repo: machine rules, build/test/UI checks, parallel work, releases, open decisions.
* `CHANGELOG.md` — what changed in each release.
* `docs/USER_GUIDE.md` — every page explained (also the in-app assistant's knowledge).
* `docs/ARCHITECTURE.md` — crates, threads, conventions.
* `site/` — the website ([uruskan.github.io/EliteNexus](https://uruskan.github.io/EliteNexus/)), published to GitHub Pages
  by `.github/workflows/pages.yml`, with `llms.txt` for AI assistants.
* `docs/research/` — verified specs: EDDN (live service differs from GitHub!), EDSM, Inara, CAPI, Spansh, websites,
  journal format, engineering data, exploration/exobiology science, HUD colours and cockpit lighting, and a
  167-feature competitive matrix.

## Licence

Elite Nexus is free software: you can redistribute it and/or modify it under the terms of the GNU General Public
License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later
version. It comes with no warranty. See [LICENSE](LICENSE). Third-party data and fonts keep their own licences: see
[NOTICE.md](NOTICE.md).

*Elite Dangerous is a trademark of Frontier Developments plc. Elite Nexus is an unofficial community tool.*
