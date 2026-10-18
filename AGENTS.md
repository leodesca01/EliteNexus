# Working on Elite Nexus

Elite Nexus is a native Rust (egui) companion for Elite Dangerous: one executable for Windows (and Linux, where the
game runs under Proton) that replaces the pile of tools and websites commanders run side by side. This file is for anyone changing the code, people and AI agents
alike. Read it before your first change; several rules protect the owner's game, data and machine.

## Read first

* `docs/ARCHITECTURE.md`: crates, threads, the design system, code conventions.
* `docs/USER_GUIDE.md`: what every page does. It's compiled into the in-app assistant's knowledge base, together
  with `crates/nexus-ai/knowledge/*.md`, so the assistant tells users whatever it says. When behaviour changes,
  update it in the same commit.
* `site/`: the website (GitHub Pages, `.github/workflows/pages.yml`) and its `llms.txt`, which AI assistants read.
  They describe features too: keep them true when behaviour changes.
* `CHANGELOG.md`: what changed in each release. Add your user-visible changes under "Unreleased" as you go; the
  release workflow publishes each version's section as its release notes.
* `docs/research/`: verified specs (EDDN, EDSM, Inara, CAPI, Spansh, the journal, engineering, exploration, HUD
  colours and cockpit lights). Check
  them before touching an integration; the live EDDN service differs from its GitHub docs, for one.

## This machine

* Windows 10, PowerShell 5.1 and Git Bash. The Rust toolchain is on `D:\GameDev\Tools\Rust` and not on PATH:
  dot-source `tools\env.ps1` (the helper scripts below do it for you).
* **The owner often plays Elite Dangerous while you work.** Build at BelowNormal priority with `-j 4`
  (`tools\dev\cargo.ps1` does both), and never open windows over the game: check the UI with off-screen
  screenshots (`tools\dev\shot.ps1`).
* **C: has only a few GB free.** Keep builds, caches and scratch files on D:.
* **The owner's real install is `dist\EliteNexus\`**, with their data in `dist\EliteNexus\nexus-data\` (settings
  with DPAPI-sealed API keys, the galaxy database, upload queues). Never read, change or delete that data. Debug
  builds keep their own in `target\debug\nexus-data\`.
* The journals in `%USERPROFILE%\Saved Games\Frontier Developments\Elite Dangerous` are read-only to us.
* The game folder (`D:\SteamLibrary\steamapps\common\Elite Dangerous\Products\elite-dangerous-odyssey-64`) is the
  owner's too: Elite Nexus changes it only when the owner clicks (HUD & cockpit › Cockpit lights). Test that code
  against a copy (`ELITE_NEXUS_GAME_DIR`). Frontier's shader code (3Dmigoto exports, frame captures) never goes
  into git; keep it under `target-agents\`.

## Linux

The game runs under Proton or Wine there, so its journals and Options sit in a Wine prefix
(`nexus_journal::paths`: `default_journal_dir`, `options_dir`, `steam_libraries`). Keep Windows-only code behind
`#[cfg(windows)]` with a Linux counterpart (`secrets.rs`, `tts.rs`, `main.rs`), and never link a system library
the Linux binary might not find at start (that's why voice runs `spd-say` instead of the `tts` crate there).

This PC has an Ubuntu 24.04 WSL1 distro for it, `EliteNexusLinux` (in `D:\WSL\EliteNexusLinux`; WSL2 would need a
kernel update with admin rights). It holds Rust, Xvfb and Mesa's software Vulkan, and a Steam/Proton layout under
`/root/.local/share/Steam` made of copies of this PC's journals, Options and control schemes. From the repository
root:

```powershell
wsl -d EliteNexusLinux -u root -- bash tools/dev/linux.sh cargo clippy --workspace --all-targets -j 4 -- -D warnings
wsl -d EliteNexusLinux -u root -- bash tools/dev/linux.sh cargo test --workspace -j 4
wsl -d EliteNexusLinux -u root -- bash tools/dev/linux.sh shot --page Settings --out linux-settings --size 1000x760
```

Builds go to `/root/target` inside the distro. CI runs the same clippy and tests on `ubuntu-latest`.

## macOS

The game runs under NotProton (Steam Play for the macOS Steam client, built on CrossOver), CrossOver or Whisky, so
macOS shares the Linux code paths (`cfg(not(windows))`); the differences are behind `cfg!(target_os = "macos")`:
the Steam folder and the bottles in `nexus_journal::paths`, the data folder (`settings::data_dir`: never inside an
app bundle), voice through `/usr/bin/say` (`tts.rs`) and the bundle's own Dock icon (`main.rs`). Prefer `cfg!` over
`#[cfg]` there, so the Linux build type-checks the macOS branches. `tools/macos/build-app.sh` builds, packs and
ad-hoc signs `Elite Nexus.app` (`--install` copies it to Applications); its icon comes from
`crates/nexus-app/examples/macos_iconset.rs`. CI runs clippy and tests on `macos-latest`.

## Hard rules

* **Never upload history**, only live events. Test runs (`--journal-dir`, `--screenshot`) never upload at all.
  Uploads stay opt-in: don't switch a service on for the owner.
* **Don't push, tag or publish** unless the owner asks (see Releases).
* Never commit `nexus-data`, databases, logs or keys (`.gitignore` covers them; don't force-add).
* Never panic on external data: no `unwrap()` on I/O or network data, `#[serde(default)]` on external structs.
* **Be honest in the UI.** Say where data comes from and what it can't show (the Exobiology bonus hunt explains why
  "no record" isn't proof; Find valuable bodies says Spansh skips populated systems). The owner asked for this
  explicitly: explain, don't just show numbers.
* **Limits must be real.** Spansh planners use `tours::limits` (Spansh's own form bounds: jump range 10-100 ly,
  radius up to 1,000 ly, up to 2,000 stops); local database searches can reach the whole galaxy. Don't invent caps;
  where a cap protects performance, say so in a tooltip.

## Build, test, check

```powershell
powershell -File tools\dev\cargo.ps1 -Log build  -CargoArgs "build -p nexus-app -j 4"
powershell -File tools\dev\cargo.ps1 -Log clippy -CargoArgs "clippy --workspace --all-targets -j 4 -- -D warnings"
powershell -File tools\dev\cargo.ps1 -Log test   -CargoArgs "test --workspace -j 4"
```

Output goes to `target\cc\<Log>.log`. CI (`.github/workflows/ci.yml`) runs the same clippy and tests on
`windows-latest` for every push to `main` and every pull request; keep both green.

* Tests that need the internet are `#[ignore]`d, e.g. `cargo test -p nexus-app untouched -- --ignored --nocapture`.
* Spansh planners, live: `cargo run -p nexus-spansh --example smoke -- --only riches,exobio` (all steps: no `--only`).
* More dev tools (journal stats, model summary, replay, EDDN upload validation) are listed in the README.

## Checking the UI

```powershell
powershell -File tools\dev\shot.ps1 -Page Exploration -Tab history -Out history -Size 1000x760
```

The window renders at -20000,-20000 (never visible, no taskbar button), saves `target\shots\<Out>.png` and exits;
open the PNG to look at it. Check a narrow (1000 px) and a wide size when a layout changes.

* `-Page`: a sidebar page (`Dashboard`, `Exploration`, `Exobiology`, `Navigation`, `Market`, `Settings`, ...).
* `-Tab`: `abnormal`, `database`, ... (Market), `galaxy` (whole-galaxy map), `find`, `history`, `unsold`
  (Exploration), `untouched`, `known` (Exobiology > Find planets), `glare`, `cockpit` or a preset name such as `mocha`
  (Hud: that tab, or HUD colours starting from that theme; add `,options` to open the protection's options; nothing
  is written). `find`, `untouched` and `known` start their search on the first frame: add `-Delay 15` to
  `-Delay 25` for Spansh to answer.
* HUD & cockpit with a game folder of your own making: set `ELITE_NEXUS_GAME_DIR` to a folder holding a dummy
  `EliteDangerous64.exe`; add a `d3dx.ini` starting `; Elite Nexus` with a `[Constants]` section (`x1 = 1` switches
  the HUD colour protection on) and an `EliteNexus-3Dmigoto.json` of `{"migoto_version":"1.4.9","files":[]}` for
  the installed state. For README shots, also point `LOCALAPPDATA` at a demo folder holding
  `Frontier Developments\Elite Dangerous\Options\Graphics\GraphicsConfigurationOverride.xml` (a theme, glare values):
  the tabs then show an applied state, and no real paths or capture leftovers. Delete the demo folders afterwards.
* `-Demo` adds a sample Ask conversation and an alert banner.

egui 0.36 layout lessons learned here:

* `Grid`: set `.max_col_width(ui.available_width())`, or wrapped text keeps a column at its first-frame width. Put
  the column that wraps last; the last column gets the width that's left.
* Settings-style rows (`widgets::Group::row`) reserve the title's width, so long values truncate.
* Don't change the zoom while a slider is dragged: the slider rescales under the pointer and runs away. That's why
  "Text and controls size" uses fixed steps.
* Pages that `app.rs` doesn't wrap in a `ScrollArea` (Exploration, Navigation, Market, Logbook, ...) scroll
  themselves: a `TableBuilder` scrolls on its own, anything else needs an explicit `ScrollArea`.
* Build pages from `widgets.rs` (`cards`, `group`, `tabs`/`segmented`, `toggle`, `primary_button`, `badge`,
  `footnote`, `empty`) and `theme.rs` (`pal`, `size`): no raw egui styling, no text below 12 pt.

## Working in parallel

* Use your own worktree or branch if you can, and always your own `CARGO_TARGET_DIR` (`target-agents\<name>`, git
  ignores it): a shared `target\` means lock waits and half-built binaries. `tools\dev\cargo.ps1 -TargetDir ...`
  and `tools\dev\shot.ps1 -Exe ...` take it.
* Files most features touch: `crates/nexus-app/src/app.rs` (page wiring), `widgets.rs`, `theme.rs`, `ai_tools.rs`
  (the assistant's tools), `pages/navigation.rs` (`PlannedRoute`), `docs/USER_GUIDE.md`. Keep edits there small,
  commit them early, and rebase often.
* Keep shared crates compiling between edits: someone may be building against them.
* Don't run `cargo fmt` over the workspace. Much of the code is hand-formatted with long lines; a mass reformat
  makes every parallel branch conflict. Match the surrounding style.

## PowerShell 5.1 traps

* `Get-Content`/`Set-Content` use the ANSI code page by default: round-tripping a UTF-8 source through them corrupts
  `·`, `›`, `≈` and friends and adds a BOM. Use an editor tool, or `[IO.File]::ReadAllText`/`WriteAllText` with
  `New-Object System.Text.UTF8Encoding($false)`. Keep `.ps1` files ASCII.
* `git commit -m "..."` breaks on embedded double quotes: write the message to a file and `git commit -F` it.
* With `2>&1`, a native command's stderr shows up as errors even on success: check `$LASTEXITCODE`.

## Commits

* One logical change per commit. Subject in the imperative, about 70 characters, often "Area: what changed"
  (`Settings: size in fixed steps; keep row titles readable`). The body says what and why, in plain words.
* Commits written by an AI agent end with a `Co-Authored-By:` trailer naming the model.
* Update `docs/USER_GUIDE.md` and `CHANGELOG.md` in the same commit as the behaviour they describe.

## Releases

1. Bump `version` under `[workspace.package]` in `Cargo.toml` (the only place it lives; the app shows it), then
   `cargo check --workspace` to refresh `Cargo.lock`.
2. In `CHANGELOG.md`, turn "Unreleased" into `## [x.y.z] - YYYY-MM-DD` and start a new empty "Unreleased".
3. Commit as `Elite Nexus x.y.z`.
4. The owner pushes with **GitHub Desktop**. The command-line git has no GitHub credentials: a push opens a
   "Credential Helper Selector" window. Don't work around it; ask the owner.
5. Tag `vx.y.z` **in GitHub Desktop** (History, right-click the commit, Create Tag, then Push origin). Desktop doesn't
   push tags made with the git command line; if one exists with that name, delete it (`git tag -d`) and have the
   owner click Fetch origin so Desktop forgets it.
6. The tag runs `.github/workflows/release.yml` (about 10 minutes on a cold cache): it builds on `windows-latest`
   and `ubuntu-22.04` (an older glibc, so the binary starts on older distributions), packs the exe and the Linux
   binary with the README, NOTICE, user guide and Inter licence (a zip and a tar.gz), and publishes the GitHub
   Release with that version's CHANGELOG section as notes.
7. Check it without credentials (the repo is public):
   `https://api.github.com/repos/uruskan/EliteNexus/releases/latest`.

The owner's own copy: `tools\build-release.ps1` (run it at BelowNormal priority with `CARGO_BUILD_JOBS=4`) installs
into `dist\EliteNexus`, or stages `EliteNexus.new.exe` while the app runs (`Update-EliteNexus.cmd` swaps it in),
and writes `dist\EliteNexus-vx.y.z-windows-x64.zip`. It never touches `nexus-data`.

## Open decisions (the owner's)

* EDGalaxyData (online footfall lookups) publishes no terms: ask its operator. The switch is
  `ai_tools::ONLINE_FOOTFALL_LOOKUP`.
* Inara whitelisted the app name "Elite Nexus" on 2026-10-13 (Artie, Inara's author), so Inara sync works with a
  user's personal key. Developer mode (`Accounts::inara_dev_mode`, sent as `isBeingDeveloped`) is still on by
  default; when to switch that default off is the owner's call, after a few weeks of clean syncs.
* `GalaxyOptions::all_eddn_bodies` (keep every landable EDDN body, not just biology and valuable worlds) is off.
* Small known issue: the EDDN "updates stored" counter slightly over-counts body updates.
* GitHub repo settings, which only the owner can change: upload `docs/social-preview.png` (Settings › Social preview),
  and set a description and topics. Suggested description: "All-in-one Elite Dangerous companion app: one fast native
  Windows exe that replaces EDDiscovery, EDMC, Observatory, Odyssey Materials Helper and your Inara/Spansh tabs.
  Exobiology, exploration, live EDDN trading, Spansh routes, engineering, fleet, carrier, an AI co-pilot, and HUD
  colours that keep hostile red." Topics: `elite-dangerous`, `elite-dangerous-odyssey`,
  `companion-app`, `hud`, `exobiology`, `eddn`, `spansh`, `trading`, `rust`, `egui`, `windows`.

## Ideas queue

* A panel showing exactly what the Ask assistant can see and do, built from its tool list (the owner's idea).
* Retake the README screenshots of Exobiology and Navigation: they predate 0.2.2's tabs. They were made from a
  replay of a real exobiology landing (`cargo run --release -p nexus-journal --example replay`). The replay writes
  only the journal: the sampling radar takes each sample's position from `Status.json` (latitude and longitude at the
  `ScanOrganic`), and the route comes from `NavRoute.json`, so a retake has to write those next to the journal too.
* A copy button next to every system name (a Reddit user's request, 0.4.0 launch thread; Inara has one). Click-to-copy
  already exists on Market, Exploration, Exobiology, Engineering, Colonisation, Galaxy map and Navigation, but it
  is invisible (a plain label with a tooltip) and each page calls `ctx.copy_text` by hand. Missing entirely: the
  Dashboard Activity feed, because `FeedItem` (`nexus-model/src/state.rs`) keeps the system name only inside `text`;
  give it a `system: Option<String>`. Plan: one shared `widgets::system_name` (name plus a copy icon, "Copied"
  flash), used on every page that shows a system; then say so in `docs/USER_GUIDE.md` and `CHANGELOG.md`. Check
  Stats, Missions, BGS, Carrier and Mining too.
* Surface compass, on-foot farming guide, installer.
