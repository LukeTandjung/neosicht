# Phase 0 — Derisking Experiments

Production-worthy experiment code was moved into the owning hexagonal layer in
`crates/neosicht`; experiment-only binaries and failed implementations were
removed. Findings remain recorded here. Experiments 0–2 are existential: all
three must pass before building the shell proper. Run them in order — a failure
earlier in the list invalidates work later in it.

Prerequisite for all experiments: a stable Developer ID signing identity. TCC
(Accessibility, Location, …) grants are tied to the signing identity;
ad-hoc-signed rebuilds lose permissions on every build and poison results.

## Experiment 0 — Non-activating panel (`exp-panel`)

The bar must never steal focus. Create a GPUI window configured as shell chrome,
reaching into the underlying NSWindow via the C shim if GPUI's `WindowOptions`
are insufficient (`NSWindowStyleMaskNonactivatingPanel`, refuse key-window
status, `canJoinAllSpaces | stationary`, explicit level).

**Pass criteria** — with TextEdit frontmost and its text field focused:

- [x] Clicking a button in our panel does not change the frontmost app.
- [x] TextEdit's text field keeps keyboard focus (typing still lands in it).
- [x] The panel is visible on every Space without animation.
- [x] The panel stays above normal app windows at its configured level.
- [x] A popover opened from the panel can receive scroll/hover without
      activating us.

**Fail →** gpui fork investigation; if fundamentally impossible, the project
needs a redesign, so this runs first.

## Experiment 1 — AX menus (`exp-ax-menus`)

CLI against the frontmost app's AX tree.

Steps: read top-level menu titles; lazily fetch a single menu's items on demand;
perform an AX press on a real item (Safari → File → New Window); observe
app-switch and re-read.

**Pass criteria:**

- [x] Top-level titles for Safari/Zed/Finder in < 50ms.
- [x] Single-menu item fetch (titles, shortcuts, enabled state) in < 100ms.
- [x] AX press executes the app's real command with the app frontmost.
- [x] Lazily-populated menus (Safari History, an "Open Recent") return usable
      items, or a documented trigger/refetch strategy works.
- [x] Electron app (e.g. VS Code) and a Java app expose workable trees.
      (Electron: pass via Notion. Java: untested — no Java app installed.)
- [x] App switch → new app's titles within 100ms (with cache).
- [x] A hung app does not block us (messaging timeout verified).

## Experiment 2 — SkyLight menu-bar suppression (`exp-skylight`)

From a normal signed process, SIP enabled: `dlopen` SkyLight, `dlsym` the
menu-bar visibility/inset symbols, capability-probe by macOS version.

**Pass criteria — menu bar stays suppressed across:**

- [ ] App switches (Safari → Zed → Finder).
- [ ] Cursor held at the top screen edge.
- [ ] Display sleep/wake and unplug/replug.
- [ ] Multiple displays (per-display behavior documented).
- [ ] A native-fullscreen app entering/leaving fullscreen.
- [ ] Mission Control in/out.

Status: **SOLVED via covering** (not via SkyLight suppression). Private-API
hiding is negative, but an opaque bar at level 25 placed at y=0 covers the
native menu bar at rest AND on hover-reveal, with zero private API. See
findings.

Also document: exact macOS versions tested, and verify the public fallback
(System Settings auto-hide "Always") as the degraded mode.

## Experiment 3 — WM coexistence (`exp-wm`)

Run the Experiment-0 panel alongside each supported window manager.

**Pass criteria per WM (yabai, AeroSpace; probe Paneru):**

- [ ] WM reserves bar space (`external_bar` / `gaps.outer.top` / Paneru
      equivalent) and tiles below the bar.
- [ ] Layout stays correct with the native menu bar suppressed (Exp 2 active) —
      the `NSScreen.visibleFrame` interaction.
- [ ] Workspace/space state readable (query) and change events received (signal
      / exec-on-workspace-change) with < 100ms pill-update latency.
- [ ] Focus commands forwarded (pill click → WM switches workspace).
- [ ] Paneru: IPC surface documented; groups+windows+focus mapping sketched.

## Experiment 4 — Dock suppression (`exp-dock`)

Low risk; do it in an afternoon. Autohide + extreme `autohide-delay`.

**Pass criteria:**

- [ ] Dock does not reveal on bottom-edge hover (nor left/right positions).
- [ ] Holds across displays, Mission Control, app activation, fullscreen
      transitions.
- [ ] Dock _process_ stays healthy (Mission Control, app switcher, minimize
      animations still work).

## Findings

Append dated findings per experiment here as they run.

### 2026-08-09 — Experiment 0: PASS

All five criteria verified manually (watch-frontmost.sh + TextEdit typing test).
**No gpui fork needed**: `WindowKind::PopUp` already produces a non-activating
shell surface out of the box. From `gpui_macos/src/window.rs` (rev `59b2ebf`):

- `NSWindowStyleMaskNonactivatingPanel` on a `GPUIPanel` (NSPanel subclass)
- `setLevel_(NSPopUpWindowLevel)` (101)
- `canJoinAllSpaces | fullScreenAuxiliary` collection behavior
- an always-active `NSTrackingArea`, so hover works while inactive

Configuration used (`crates/neosicht/src/ui/bar.rs`): `kind: PopUp`,
`titlebar: None`, `focus: false`, `is_movable/resizable/minimizable: false`, and
— critically — no `cx.activate(true)` anywhere.

Caveats / follow-ups:

- Window level 101 is too high for a real bar (sits above context menus and some
  system UI). Proper leveling (`kCGStatusWindowLevel`-ish) will need a
  post-creation `setLevel:` through the C shim — one call, not a fork.
- Popover windows currently need manual screen-coordinate placement; fine for
  the shell (we know the bar geometry).

Build-environment findings (macOS 26 / Xcode 26.6 / Nix dev shell):

- gpui's build script compiles `shaders.metal` via `xcrun` from PATH. The Nix
  apple-sdk `xcrun` cannot drive Xcode 26's downloadable Metal Toolchain, even
  with `DEVELOPER_DIR` set — shader compilation fails with "missing Metal
  Toolchain".
- Fix used: enable `gpui_macos`'s `runtime_shaders` feature (shaders compile at
  app startup). Release builds can switch back using
  `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer /usr/bin/xcrun`.
- The Metal Toolchain itself must be downloaded once:
  `xcodebuild -downloadComponent MetalToolchain` (~688 MB).

### 2026-08-09 — Experiment 1: PASS

`crates/neosicht/src/adapters/accessibility.rs` — pure C FFI (raw `extern "C"`
AX declarations + `core-foundation` crate), no Obj-C, no gpui. Measured
(M-series, debug build):

| Operation                                                   | Result                 |
| ----------------------------------------------------------- | ---------------------- |
| Top-level titles (Finder 8, Safari 9, TextEdit 8, Notion 8) | 10–15 ms               |
| One menu's items incl. enabled + shortcuts (2–43 entries)   | 2–18 ms                |
| AXPress Safari File → New Window                            | AXError 0 in 3.6 ms    |
| Full recursive tree (Safari, 468 elements)                  | 207 ms                 |
| Suspended (SIGSTOP) app query, 1.0s messaging timeout       | fails at 1.02 s, clean |

Key findings:

- **The lazy-menu fear did not materialize**: Safari's History menu is fully
  populated in the AX tree (real history entries, dated submenus) without the
  menu ever being opened. No trigger/refetch strategy needed for it.
- **AXPress works on a non-frontmost app** and does not activate it — stronger
  than the criterion. The shell can drive menu commands without focus juggling.
- Full-tree walks (207 ms) confirm the plan: fetch per-menu on demand (2–18 ms),
  never walk the whole tree on the interactive path.
- Electron (Notion) exposes a complete native menu tree. Java untested — no Java
  app on this machine; revisit when one is at hand.
- Quirk: the system-wide element's `AXFocusedApplication` attribute returns
  kAXErrorCannotComplete (-25204) from this CLI context even with Accessibility
  granted (sandboxed and unsandboxed), while System Events reads focus fine.
  Frontmost-app tracking in the shell should use the NSWorkspace shim (planned
  anyway) or `CGWindowListCopyWindowInfo`; do not depend on system-wide
  `AXFocusedApplication`.
- Accessibility TCC attributes to the hosting terminal during development; the
  packaged shell app will need its own grant (see prerequisite above).

### 2026-08-09 — Experiment 2: NEGATIVE (private-API suppression) / covering deferred

Tested on macOS 26 (M-series, notched built-in display), SIP enabled, from an
ordinary process. The experiment runtime-resolved all relevant `SLS*` symbols
via dlopen/dlsym (never linked); all symbols were present and callable. Its
implementation was removed after the covering approach succeeded.

What was tried, and what actually happened on screen:

| Lever                 | Symbol                                                                              | Result                                                                                                                                                                  |
| --------------------- | ----------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Visibility override   | `SLSSetMenuBarVisibilityOverrideOnDisplay`                                          | Only forces the bar VISIBLE. With autohide on, value 1 reveals the hidden bar (verified). No value (0/1/2/3) hides an always-visible bar. There is no "hide" direction. |
| System override alpha | `SLSSetMenuBarSystemOverrideAlpha(cid, 0)`                                          | No visible effect, even re-applied at 60Hz.                                                                                                                             |
| Max reveal cap        | `SLSTransactionSetMenuBarOverrideMaximumReveal` via `SLSTransactionCreate`/`Commit` | Commit returns a garbage (pointer-like) value → transaction ABI is wrong; hover still reveals normally.                                                                 |

Conclusion on the research doc's central question — _can a normal SIP-enabled
process keep Apple's menu bar suppressed?_ — **not with the obvious SLS levers
on macOS 26.** The override is a reveal (show) control, not a hide control.
Suppression via these APIs looks negative; a working path would require getting
the transaction ABI right (uncertain, undocumented) or a different mechanism.

**Covering path (recommended, but blocked):** our own opaque bar renders at
window level 101 (NSPopUpWindowLevel) vs the native menu bar's level 24, so in
principle our bar can just cover the native one with zero private API. But
`CGWindowListCopyWindowInfo` shows AppKit **clamps the exp-panel window to
y=32** (below the menu-bar band), so it never overlaps the native bar — the
covering test is invalid until the window is forced to y=0. That needs
post-creation NSWindow positioning (the C-shim surgery already anticipated in
Exp 0), plus notch-aware height/layout. Deferred by decision — revisit alongside
the notch design work.

Notch note: on the built-in display the menu-bar band is ~37px and the
WindowServer reserves y=0..83 (notch + bar); the shell bar must be sized/placed
for this region, not the 30px content-area default.

Next-step options recorded: (a) NSWindow-position the bar to y=0 and re-run the
covering test (most promising, no private API); (b) keep digging on the
transaction ABI for max-reveal; (c) accept public autohide as the degraded
fallback. Not blocking Exp 3/4.

### 2026-08-09 — Experiment 2, part 2: SOLVED via covering (no private API)

After the SkyLight suppression dead-end, the covering approach works — the
blocker was only window placement, now defeated. Reference apps that informed
this: Barik (replace the menu bar with your own bar) and TopBounce (clamp the
cursor so the native bar never reveals).

Winning recipe (all public API, in `crates/neosicht/src/ui/bar.rs` +
`native/pin.m`):

1. gpui `WindowKind::PopUp` bar-height window (non-activating, from Exp 0).
2. Tiny Obj-C shim compiled via `cc` (the first `neosicht-sys` seed), exporting
   plain C — Rust never touches Obj-C objects:
   - **Override `-[NSWindow constrainFrameRect:toScreen:]` to return the frame
     unchanged**, via `class_replaceMethod` on gpui's `GPUIPanel` class. This is
     a runtime method replacement scoped to our OWN process's window class — no
     injection into other processes, SIP-safe. Without it, AppKit clamps the bar
     to y=32 on a notched display (the notch/menu-bar strip is hard-reserved);
     with it, `setFrame` places the bar at y=0.
   - Set window level to `kCGStatusWindowLevelKey` (25), just above the native
     menu bar (24), then `setFrame` to the top strip.
   - `neosicht_menu_bar_height()` returns the exact band height
     (`frame - visibleFrame` top inset = 32px on this display) so the bar covers
     the notch band without spilling below.

Verified: `CGWindowListCopyWindowInfo` reports the bar at
`layer=25 x=0 y=0
w=1512 h=32`. On screen, the bar sits flush at the very top,
and **moving the cursor to the top edge does not reveal the native menu bar** —
because our opaque bar at level 25 covers the native bar (level 24) even as it
reveals underneath. Clicks in the top strip hit our bar, not the native one.

Consequences:

- **TopBounce cursor-clamp is NOT required** for basic suppression — covering
  handles both rest and hover. Keep it in reserve for edge cases (e.g. the
  native bar peeking during Mission Control / fullscreen transitions).
- Menu-bar suppression is **no longer an existential risk** — it's a solved
  covering problem, pending edge-case testing (native fullscreen, multi-display,
  external non-notched displays).
- The `constrainFrameRect` override + level-25 placement is a reusable
  `neosicht-sys` primitive the real shell bar will use.

Still to verify later (not blocking): native-fullscreen spaces, multiple
displays, external displays without a notch, Mission Control.

### 2026-08-09 — Experiment 2, part 3: key-focus fix

After moving the bar to level 25 + y=0, clicking it dropped the focused app's
text cursor — a subtler focus steal than Exp 0 tested. Cause: gpui's `GPUIPanel`
returns `canBecomeKeyWindow = YES`, so clicking the bar makes it the key window
and the previously focused text field resigns key (cursor drops), even though
the app stays frontmost (so `lsappinfo front` still looked clean).

Fix (in `native/pin.m`, same in-process class override): `class_replaceMethod`
`GPUIPanel`'s `canBecomeKeyWindow` and `canBecomeMainWindow` to return NO. A
non-activating window still receives mouse clicks without being key (like the
real menu bar / Dock), so the bar's buttons keep working. Verified: clicking the
bar no longer disturbs the focused app's cursor, and clicks still register.

Note for the shell: overlays that DO need keyboard input (launcher / command
palette) must not be plain `GPUIPanel`s under this override — make them a
`Normal` window or add a targeted opt-in. Not an issue for the always-on bar.

### 2026-08-09 — Experiment 3 (AeroSpace): confirmed in live use

Observed on the test machine already running AeroSpace 0.20.3-Beta with the
exp-panel bar active — no interference.

- **Coexistence**: bar at level 25 / y=0–32 with `canJoinAllSpaces`; AeroSpace
  tiles windows at y=48 (visibleFrame top inset 32 + its `gaps.outer.top` 16),
  clear of the bar. Nothing to do — the menu-bar band is already macOS-reserved.
- **Bar-space reservation** is a standard WM config knob (`gaps.outer.top`), not
  our concern — every serious macOS WM exposes it. Setup docs will just say "set
  the top gap to the bar height." Not a risk.
- **Provider read path**: `aerospace list-workspaces --all/--focused` and
  `aerospace list-windows --focused` return usable state via CLI.

Still to do for Exp 3 (not blocking): event push (`exec-on-workspace-change`),
focus-forward command (`aerospace workspace N`), then yabai and Paneru.
