# Changelog

What changed in each release, newest first. Add user-visible changes under "Unreleased" as you make them; at
release time that section becomes the new version and the release workflow publishes it as the release notes.

## Unreleased

### Added
- **macOS** (Apple Silicon and Intel): `tools/macos/build-app.sh --install` builds `Elite Nexus.app` on your Mac
  (Metal rendering, its own icon, an ad-hoc signature) and puts it in Applications. The game runs under NotProton
  (Steam Play for the macOS Steam client), CrossOver or Whisky; Elite Nexus finds its journals in the game's prefix
  in every Steam library (NotProton keeps Proton's `compatdata` layout), then in CrossOver's bottles and `~/.wine`,
  and in Whisky's bottles only when none of those has one (macOS asks before an app looks into Whisky's container).
  Data lives in `~/Library/Application Support/EliteNexus`, never inside the app. Voice alerts use the system's
  `say`. API keys aren't encrypted on macOS (no DPAPI), and the app says so. Cockpit lights under CrossOver is
  untested; the tab names the Steam launch option it may need.

## [0.5.0] - 2026-10-17

Elite Nexus now runs natively on Linux, next to Elite Dangerous under Proton or Wine.

### Added
- **Linux build** (`EliteNexus-…-linux-x64.tar.gz`, 64-bit; X11 or Wayland, Vulkan or OpenGL). The game runs under
  Proton or Wine; Elite Nexus finds its journals in the game's Proton prefix in every Steam library (native, Flatpak
  and Snap Steam), then in Lutris's and Heroic's usual prefixes. For any other prefix, set Settings › Journal ›
  Custom journal folder: Bindings and HUD colours then use the Options folder in that same prefix. Voice alerts use
  speech-dispatcher's `spd-say`. API keys aren't encrypted on Linux (no DPAPI), and the app says so: the settings
  file is readable only by your user account. Cockpit lights under Proton is untested; the tab names the Steam
  launch option it may need. Settings › About › App menu (or `./EliteNexus --install-desktop`) adds Elite Nexus and
  its icon to your desktop's app menu, in your own folders. On Wayland the compact HUD can't stay on top (Wayland
  doesn't allow it); its button says how to run through XWayland instead. Built and tested on Ubuntu; not yet tried
  with the game itself under Proton.

### Changed
- Bindings finds the game's built-in control presets in every Steam library and the Epic and Frontier folders,
  instead of four fixed Windows paths.

## [0.4.0] - 2026-10-13

A rebuilt Market (a page for every commodity, pins, a run planner, lists you sort by clicking a heading), a readable
Statistics page, and a cockpit-light dimmer that no longer runs patterns inside the game, which could crash it near
the newer fleet carriers.

### Added
- Market: click a column heading (price, demand, distance, updated, ...) to sort any list by it; click again to
  reverse it, a third time for the usual order. Each list remembers how it was sorted.
- **Market › Find commodity is a commodity page**: suggestions as you type ("ltd" finds Low Temperature Diamonds,
  "thar pro" Thargoid Probe, "salvage" lists all salvage), the galactic average, how many markets sell and buy it,
  and three lists: where to buy, where to sell, and the latest prices of every market trading it. Click a row for that
  market's details and this commodity's price history there. An empty list explains why ("Nobody in range buys
  Thargoid Probe"), names the nearest market your filters left out and widens them in one click.
- **Pins**: pin any entry (commodity lists, Abnormal prices, a station's market) and it stays on top of the Market
  page with its latest price, distance and age. In a pinned fleet carrier's system, the pin says whether the carrier
  is really there, from the carriers the game listed since you arrived; the Station tab lists them with their markets.
- **Market › Run planner** fills the pinned orders: where to buy what each pinned buyer wants (cargo aboard first),
  a buyer for each pinned seller, what fits the hold and credits, and the stops in an order that buys before it
  sells. Follow this run puts the stops on the Navigation page.
- Clicking a commodity anywhere on the Market page (Abnormal prices, trade routes, Sell my cargo, a station's market)
  opens its page; clicking a station opens its whole market on the Station tab (formerly This station).

### Changed
- **Statistics** reads at a glance: pick 7 days, 30 days, 90 days or all time and every chart fits that period;
  amounts show as B and M instead of "20000 M Cr"; income and spending are one row per category with their amounts
  (a 30 B trade total no longer flattens the rest), moving credits to your own carrier isn't counted as spending, and
  the credits chart has a log scale for climbs from millions to billions.
- Market remembers your last commodity, recent ones, filters, pins and tab between runs. Find commodity re-runs as
  you change a filter, and its defaults are wider: 100 ly, any quantity.
- Price alerts: the commodity field suggests names as you type.
- **Inara sync works**: Inara approved the app name "Elite Nexus" on 13 October 2026, so your personal Inara API key
  is all it needs (0.3.0 already sends the right name). Accounts & sync no longer reads as if approval is pending,
  and still explains if Inara ever refuses the app.
- Rebuilt the GitHub Pages website with a complete feature directory, dedicated exploration, commander and
  companion pages, and trading content focused on the app's trading tools. Screenshots now open in a full-size
  viewer with zoom and keyboard navigation. Added responsive navigation, a product tour and clearer setup guidance.

### Fixed
- **HUD & cockpit › Cockpit lights could crash the game** when it loaded the newer fleet carriers: the dimmer (and
  the HUD colour protection) ran a pattern inside the game on every shader it loaded. Now a one-time **calibration**
  (start the game, get into your cockpit, quit) finds the shaders it needs among the ones your game loaded, and in
  the game 3Dmigoto only swaps exactly those, by fingerprint; anything else, such as a new carrier, is left alone.
  The tab shows which game build it was calibrated for and asks for a new calibration after a game update (until
  then the dimmer is simply off). Installs from 0.3.0 get **Update and calibrate**, which also puts back a
  d3d11.dll renamed to switch the hook off.
- Accounts & sync: an upload error a service has recovered from since (it accepted uploads again) now shows dimmed,
  with its age, instead of staying red until the next error.
- Market: stations listing a good they produce (the game gives it a token demand of 1, "no demand") no longer show
  as buyers of it in searches, Sell my cargo, trade routes or alerts. Existing data is corrected once on start.
- Market › Find commodity no longer finds nothing without saying why: an unknown name says so, and a search with
  no results shows what is out of range.
- Accounts & sync › EDAstro: uploads no longer pause with "unreadable response" when EDAstro writes its reply's
  status as text (`"200"`), the way EDMC already accepts it; any reply that still can't be read now shows what
  EDAstro sent.
- Accounts & sync › Inara: "Check again", or a new key, within 35 seconds of the last Inara request no longer
  leaves the check spinning until your next sync; it now goes out as soon as Inara's request spacing allows.

## [0.3.0] - 2026-10-13

The HUD & cockpit page: HUD colours that keep hostile red, a cockpit-light dimmer, and glare and exposure. (The
v0.2.3 download already had these, but still called itself 0.2.2 and had no release notes.)

### Changed
- **Elite Nexus is now free software** under the GNU General Public License, version 3 or later: use it, share it
  and change it, as long as what you pass on stays under the same licence with its source. The licence is in the
  download (`LICENSE`), and Settings › About says so.
- Each release now has a `.sha256` file next to the zip, so you can check your download
  (`Get-FileHash EliteNexus-…-windows-x64.zip` in PowerShell shows the same value).

### Added
- **HUD & cockpit** (System), tab **HUD colours**: recolour or dim the HUD with the game's own colour setting. Start from the standard
  orange or a community theme (credited to its author), turn the colour wheel, lower saturation or brightness, or set
  the nine matrix numbers; a live cockpit preview shows the result, the swatches show what hostile red, friendly
  green and shield blue become, and the page warns when friend and foe would look alike or the HUD gets too dark.
  Applying edits only the colour lines of `GraphicsConfigurationOverride.xml`, keeps everything else in it, never
  touches a malformed file, and copies your original aside first. Share themes as the `<MatrixRed>` lines forums use,
  and save your own. The page explains what the setting can't do (cockpit lights, single elements) and warns when
  EDHM is installed, since EDHM needs it left at the standard colours.
- **HUD & cockpit › Glare and exposure**: Odyssey's auto-exposure darkens everything outside when the cockpit lights
  fill the view, and it can't be switched off. This tab sets what players change instead, in the same override
  file: glare from lights (`GlareCompensation`), the standard ACES tone curve (the page shows how much brighter darks
  and mid-tones get) and the auto-exposure targets, with presets for the game's values, less glare, and CMDR
  Morbad's 2021 Odyssey settings. The game's own values are read from its install.
- **HUD & cockpit › Cockpit lights**: dim the cockpit's own lights, which the game has no setting for. Elite Nexus
  installs 3Dmigoto (downloaded from its official release and checked against pinned checksums) with a pattern
  that adjusts the game's light shader as it loads, so lights within a reach of your eyes keep only part of their
  light; stations, carriers, planets and the HUD are untouched. Set the light kept, the reach and on/off in the
  tab or with F9, F10 and F11 in the game (the tab warns when your controls use one of those keys too); Remove
  takes out exactly what was installed, and Delete them clears frame captures and exported shaders left over from
  setting 3Dmigoto up. The game's code is never copied, so most game updates don't break it. Leaves EDHM and other
  3Dmigoto mods alone.
- **HUD & cockpit › HUD colours: Keep hostile red, friendly green and shield blue.** The game's colour matrix moves
  every HUD colour together, so a blue theme turned hostile contacts blue too. With Elite Nexus's graphics hook
  (installed from Cockpit lights), a pattern on the HUD shaders that apply the matrix applies it only to the HUD's
  orange: hostile red, friendly green, shield blue and white keep their colours. The preview, swatches and warnings
  follow the same rule; the thresholds sit under Options, because Frontier documents none of this.

## [0.2.2] - 2026-10-12

### Added
- **Exploration › Find valuable bodies**: Spansh's Road to Riches, moved here from Navigation. Spansh skips
  populated systems, so a search from the bubble used to come back empty; it now widens itself (100, 250, 500,
  then 1,000 ly) until it finds something and says so. Pick Earth-likes, ammonia worlds or rocky and metal-rich
  worlds, how far, how many stops, and whether you'll map planets.
- **Exploration › History**: every system you scanned and what became of the data: sold (and when), unsold, not
  sellable, or lost with your ship. Data sales are recorded.
- **Exobiology › Find planets** with two strategies side by side. **Bonus hunt** looks for planets with no record of
  anyone sampling them and spells out what it found: how many planets suit the species, how many already have
  samples on record, were mapped without the genus, are in populated systems or were landed on. Filters for
  evidence, chance, distance from the star and gravity apply instantly, and **Tour the best 15** plans a route
  through them. **Sure value** (moved from Navigation) lists planets where commanders already logged valuable
  species: certain, but expect the base value.

### Changed
- Navigation keeps the travel planners (neutron, exact, fleet carrier) and links to the pages that plan scan
  tours; every tour, and every route the assistant plans, shows up there.
- "Mapped, no samples" planets (formerly "Mapped, not sampled") are hidden by default, with the reason: a sample
  only leaves a record when it adds a codex entry for that commander.
- Limits follow what the services really allow: Spansh planners take jump ranges of 10-100 ly, a radius up to
  1,000 ly and up to 2,000 stops; market searches reach the whole galaxy; trade routes cover up to 1,000 ly.
- A system you haven't scanned, or a populated one, explains itself on the Exploration page instead of showing
  zeros.
- The assistant's money routes widen their search the same way and say that exobiology values are base values.

### Fixed
- A planned route that starts and ends at your current system (every Road to Riches tour) counted as finished the
  moment it was planned; stops now tick off as you arrive.

## [0.2.1] - 2026-10-12

### Added
- **Exobiology › Find untouched planets**: planets where a valuable species should grow (Stratum Tectonicas by
  default) that no commander sharing data has mapped or sampled, so the first-logged bonus (×5) is likely still
  there. The assistant got the matching `untouched_biology` tool (15 tools).

### Changed
- Accounts & sync explains that Inara accepts only apps it has approved by name, with a link to ask for approval.

### Fixed
- Settings › Text and controls size: the slider ran away to 0.75× or 1.75× while dragged; it now comes in fixed
  steps (90-175 %), and Ctrl + / Ctrl − / Ctrl 0 change it too.
- Long values such as folder paths no longer squeeze a settings row's title to one letter per line.
- The release zip includes the Inter font licence.

## [0.2.0] - 2026-10-12

### Added
- **Market › Abnormal prices**: stations and fleet carriers paying far above (or selling far below) the galactic
  average, 1.5× to 10×, across the whole galaxy or a radius.
- A real **galaxy map**: a painted Milky Way with your travels, route, ship and carrier, search, layers and "Plot
  route" to any system.
- Planets: a footfall-aware bodies table, and the assistant's `system_bodies` tool ("has anyone set foot on
  planet X?").
- Inara: the key check shows which commander it belongs to, and the profile is sent as soon as sync starts.

### Changed
- A redesign after macOS 26: tinted cards, grouped settings rows, segmented controls, a floating glass sidebar and
  a toolbar capsule; every page adapts to the window width.
- The interface is set in Inter, bundled with the app.

## [0.1.0] - 2026-10-12

First release: journal-driven dashboard, exploration, exobiology (sampling radar and species predictions), a
live EDDN market database (commodity search, sell my cargo, trade routes, services, hotspots, price alerts),
Spansh route planners, fleet, engineering, missions, mining, fleet carrier, colonisation, BGS, logbook,
statistics, bindings, opt-in EDDN / EDSM / Inara / EDAstro uploads, voice alerts, a compact HUD and the Ask
assistant (Gemini, Claude or a local model).
