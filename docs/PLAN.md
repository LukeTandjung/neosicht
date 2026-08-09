# neosicht — Plan

An alternate desktop shell for macOS in Rust + GPUI. macOS keeps WindowServer,
`loginwindow`, and normal `.app` applications; neosicht replaces the visible
shell chrome: menu bar, application menus, workspace indicator, and status
widgets.

## Hard constraints

- **SIP stays fully enabled.** No process injection, no debugger attachment to
  system processes, no modification of protected binaries. Private APIs are
  permitted only when callable from our own ordinary process.
- **No window management of our own (v1).** neosicht must coexist with yabai,
  AeroSpace, and Paneru. Window/workspace state is read from the running window
  manager through a provider port; commands (focus workspace, etc.) are
  forwarded to it. A built-in AX window manager is a possible _later_ provider
  backend, never a v1 goal.
- **Rust never touches Objective-C objects.** All macOS integration crosses a
  plain C ABI (POD structs, `extern "C"` functions). Unavoidable AppKit
  functionality is wrapped in minimal Objective-C that exports C functions.
- **Private APIs are quarantined.** If a private API is introduced, its symbols
  must be resolved at runtime behind a capability-probed adapter with graceful
  degradation. The failed SkyLight experiment is documented but not shipped.

## The design (v2)

Source of truth: Claude Design project "macOS Desktop Environment Design"
(`Desktop Environment v2.dc.html`). The design is **bar-only** — no dock, no
launcher, no overview. One top bar (~30px) containing:

- **App menu chip** — frontmost app's name; opens a single popover with a menu
  rail (File/Edit/View/…) on the left and the hovered menu's items on the right.
  Backed by the Accessibility (AX) menu tree of the frontmost app; clicking an
  item performs the AX press so the app executes its real command.
- **Workspace pills** — numbered groups with per-app icons, backed by the active
  window-manager provider.
- **Center island** — weather + now-playing EQ + clock; opens a panel with
  weather/forecast, calendar month + events, and now-playing transport.
- **Right cluster** — notifications, audio output/volume, Bluetooth, Wi-Fi (with
  WPA2 join modal), battery, wallpaper picker, and a base16 themer (8 palettes ×
  dark/light, font sets, accent slot).

UI components come from **base-gpui** (separately maintained, BaseUI-style
headless component library for gpui). Shell-specific machinery — the 16-slot
theme engine, non-activating panel windows — stays in this repo.

## Findings that shape the plan

### Existential risks (all have experiments; see docs/EXPERIMENTS.md)

1. **Focus stealing.** If clicking the bar activates our process, the "frontmost
   app" we proxy menus for becomes _us_ and the illusion collapses. The bar must
   be a non-activating panel (`NSWindowStyleMaskNonactivatingPanel`, never key
   window, `canJoinAllSpaces | stationary` collection behavior, explicit window
   level). GPUI is built for app windows; this may require a small gpui fork or
   post-creation NSWindow surgery via the C shim.
2. **AX menu fidelity.** Every AX attribute read is synchronous IPC into the
   target app. Full-tree walks are too slow (seconds for large apps) and many
   apps populate menus lazily (`menuNeedsUpdate:` — Safari History, "Open
   Recent"). Mitigation: the chip-popover design needs only top-level titles
   eagerly and one menu's items on rail hover; cache per app, invalidate via
   `AXObserver`, set `AXUIElementSetMessagingTimeout`, do all AX work off the
   render path.
3. **Menu-bar suppression.** SkyLight suppression failed on macOS 26 and its
   experimental implementation was removed. The working approach uses the public
   AppKit panel adapter to cover the native bar at level 25. It still needs
   verification across displays, fullscreen, and Mission Control.

### Window-manager coexistence

Prior art: SketchyBar. Integration surfaces:

| WM        | State                                                         | Events                                      | Bar space               |
| --------- | ------------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| yabai     | `yabai -m query --spaces/--windows/--displays` (JSON, socket) | `yabai -m signal --add …` invoking our hook | `external_bar main:H:0` |
| AeroSpace | `aerospace list-workspaces/list-windows` (CLI)                | `exec-on-workspace-change`                  | `gaps.outer.top`        |
| Paneru    | TBD — scrolling columns, IPC surface to be probed             | TBD                                         | TBD                     |

- Ship a `neosicht-msg` CLI/unix-socket as the universal push interface
  (SketchyBar `--trigger` model); poll queries as fallback.
- Provider port models abstract _groups + windows + focus_, not "workspace
  number" — Paneru's scrolling-column model has no discrete workspaces.
- Novel combination to test: each WM's layout math (`NSScreen.visibleFrame`)
  with the native menu bar suppressed.

### Widget feasibility map

| Widget                | API                                                                                                                                                                      | Risk        |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------- |
| Clock, calendar grid  | local time                                                                                                                                                               | none        |
| Battery               | `IOPSCopyPowerSourcesInfo` (public C)                                                                                                                                    | none        |
| Audio volume + output | CoreAudio (public C)                                                                                                                                                     | none        |
| Wallpaper             | our own desktop-level window                                                                                                                                             | none        |
| Weather               | HTTP (e.g. Open-Meteo; WeatherKit needs entitlement — skip)                                                                                                              | none        |
| Calendar events       | EventKit (Obj-C shim; Calendar TCC)                                                                                                                                      | low         |
| Wi-Fi list/join       | CoreWLAN (Obj-C shim); **SSID scan needs Location Services since Sonoma**; join via `associate`                                                                          | medium      |
| Bluetooth             | IOBluetooth device list public-ish; **power toggle only via private `IOBluetoothPreference*`** (blueutil); AirPods battery is private/BLE                                | medium–high |
| Now playing           | MediaRemote is private **and entitlement-gated since macOS 15.4** (broke nowplaying-cli). v1: per-app scripting (Music/Spotify). MediaRemote-adapter trick as experiment | high        |
| Notifications         | **No API.** NC AX-scraping or NC sqlite db (Full Disk Access, schema churn). DND via `shortcuts` CLI + Focus shortcut. v1: shell-generated events only                   | highest     |

Rule: the bell and the EQ bars must never block the bar. They degrade.

### Permissions inventory

Accessibility (menus, windows), Location (Wi-Fi scan), Calendar (events),
possibly Full Disk Access (notifications, later). Enough prompts that a
first-run onboarding surface is part of the product.

**Prerequisite:** stable Developer ID signing from day one — TCC grants are tied
to the signing identity, and ad-hoc-signed rebuilds lose Accessibility
permission on every build.

## Architecture

Hexagonal, one Cargo workspace, with cohesive product capabilities as crates
under `crates/`. Crates are not split by architectural layer or implementation
technology. Each feature or widget crate follows the same internal architecture
where those layers earn their existence.

The existing shell foundation and all Phase-0 experiments are consolidated in
one crate:

```text
crates/neosicht/
└── src/
    ├── ports/           # one safe contract per FFI capability
    ├── app/             # wires ports into operations consumed by UI
    ├── adapters/        # flat Rust adapters and their native build assets
    ├── ui/              # GPUI bar consuming app operations
    ├── impls/           # composition root: adapters → app → UI
    └── main.rs
```

Future widgets and features become their own cohesive crates—menus, workspaces,
status widgets, and similar capabilities—rather than separate core, UI, FFI, or
adapter crates. For example, Accessibility menu discovery belongs to a future
menu-feature crate, not `neosicht`. Each crate follows the same internal
hexagonal structure, adding a core layer only when it has pure domain decisions.
`neosicht` has no core yet.

Ground rules:

- Each FFI capability has a corresponding safe port. Raw C declarations and
  `unsafe` are implementation details of that port's adapter.
- `app` depends on ports; adapters implement ports. `impls` is the only layer
  that imports app and concrete adapters, and it composes them with UI.
- FFI is not centralized merely because it is FFI. C/Objective-C sources and
  safe Rust wrappers stay with the feature whose adapter owns them.
- A port earns its existence only when alternatives, a test double, or graceful
  degradation justify the abstraction.
- UI is an explicit outer layer that maps GPUI interactions to application use
  cases and renders application state.
- Async runs on GPUI's executors; no tokio in the shell. Adapters push events
  through the owning crate's application event path.
- Crate boundaries represent cohesive product capabilities, not dependency
  direction by themselves; module boundaries enforce the hexagonal direction.

Application layers are unit-tested against in-memory adapters; adapter behavior
is covered by focused integration tests. Experimental findings remain in
`docs/EXPERIMENTS.md`, not in production binaries.

## Phases

Phase 0 is the derisking gate — see `docs/EXPERIMENTS.md`. Experiments 0–2
passing means the project is engineering, not research.

Each subsequent phase ships a usable bar:

1. **Skeleton** — consolidate experiments into `neosicht`, establish its
   internal hexagonal modules, and promote the non-activating panel into the
   empty themed GPUI bar.
2. **Zero-risk widgets** — clock, battery, audio, wallpaper/desktop surface,
   base16 theme engine. Public C APIs only; forces the ports/adapters plumbing
   into existence.
3. **Menu chip** — `svc-menus` + `adapter-ax` + the rail/items popover.
4. **Workspace providers** — AeroSpace adapter first (cleanest CLI), then yabai,
   then `neosicht-msg`, then Paneru.
5. **Medium-risk widgets** — Wi-Fi, calendar events, weather; permissions
   onboarding surface.
6. **Hard widgets (deferrable)** — Bluetooth toggle, now-playing (scripting
   first, MediaRemote adapter as experiment), notifications last (bell renders
   shell-generated events only until the NC experiments say otherwise).

## Non-goals

Replacing WindowServer, `loginwindow`, the Cocoa application model, system
authorization UI, application rendering, or native fullscreen semantics.
Preferred "fullscreen" is maximize-to-usable-bounds. No App Store distribution
(private APIs preclude it).
