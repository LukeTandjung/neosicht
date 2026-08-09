//! Experiment 2 — SkyLight menu-bar suppression.
//!
//! Private `SLS*` symbols are resolved at RUNTIME via dlopen/dlsym only — we
//! never link them. This is the quarantine pattern from docs/PLAN.md: nothing
//! outside this crate names an `SLS*` symbol, and every capability is probed
//! before use so an OS change degrades gracefully instead of failing to load.
//!
//! Signatures for these functions are reverse-engineered (yabai, Barik, etc.)
//! and are NOT guaranteed correct on any given macOS. So the tool is built to
//! OBSERVE: `probe` is read-only and prints resolved symbols + current state;
//! only run a mutating subcommand after probe looks sane.
//!
//! Safety: every mutating subcommand auto-restores the previous state after a
//! timeout and on Ctrl-C, so a wrong guess cannot strand the menu bar.
//!
//! Subcommands:
//!   probe                     read-only: resolve symbols, dump per-display state
//!   override <hide|show> [s]  SLSSetMenuBarVisibilityOverrideOnDisplay on all displays
//!   autohide <on|off> [s]     SLSSetMenuBarAutohideEnabled (connection-wide)
//!
//! [s] = seconds to hold before auto-restoring (default 8).

use std::ffi::{CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ports::skylight::{DisplayId, SkyLight as SkyLightPort, SkyLightError};

// ---------------------------------------------------------------------------
// Public CoreGraphics display helpers (these ARE public C API)
// ---------------------------------------------------------------------------

type CGDirectDisplayID = u32;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGMainDisplayID() -> CGDirectDisplayID;
    fn CGGetActiveDisplayList(
        max_displays: u32,
        active: *mut CGDirectDisplayID,
        count: *mut u32,
    ) -> i32;
}

fn active_displays() -> Vec<CGDirectDisplayID> {
    let mut list = [0u32; 16];
    let mut count: u32 = 0;
    let err = unsafe { CGGetActiveDisplayList(list.len() as u32, list.as_mut_ptr(), &mut count) };
    if err != 0 {
        eprintln!("CGGetActiveDisplayList failed: {err}; falling back to main display");
        return vec![unsafe { CGMainDisplayID() }];
    }
    list[..count as usize].to_vec()
}

// ---------------------------------------------------------------------------
// Runtime-resolved private SkyLight symbols
// ---------------------------------------------------------------------------

type CGError = i32;

// Reverse-engineered signatures. `cid` is the main connection id.
type FnMainConnectionID = unsafe extern "C" fn() -> i32;
// The override is an INT enum, not a bool: earlier probing showed value 1 =
// force-visible. Candidates: 0 = no override, 1 = force visible, 2 = force
// hidden. We read/write it as i32 so we can sweep the enum space.
type FnGetOverride = unsafe extern "C" fn(i32, CGDirectDisplayID, *mut i32) -> CGError;
type FnSetOverride = unsafe extern "C" fn(i32, CGDirectDisplayID, i32) -> CGError;
type FnGetAutohide = unsafe extern "C" fn(i32, *mut bool) -> CGError;
type FnSetAutohide = unsafe extern "C" fn(i32, bool) -> CGError;
type FnMenuBarExists = unsafe extern "C" fn(i32) -> bool;
// Alpha lever: force the whole menu bar transparent regardless of its
// visibility/autohide state. Reset clears the override.
type FnSetOverrideAlpha = unsafe extern "C" fn(i32, f64) -> CGError;
type FnResetOverrideAlphas = unsafe extern "C" fn(i32) -> CGError;
// Reveal lever (transaction only): cap how far the bar can reveal on hover.
// Setting max reveal to 0 with autohide on = hover shows nothing.
type FnTransactionCreate = unsafe extern "C" fn(i32) -> *mut c_void;
type FnTransactionSetMaxReveal = unsafe extern "C" fn(*mut c_void, f64);
type FnTransactionCommit = unsafe extern "C" fn(*mut c_void, i32) -> CGError;

const SKYLIGHT_PATH: &str = "/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight\0";

pub struct SkyLightAdapter {
    handle: *mut c_void,
    main_connection_id: Option<FnMainConnectionID>,
    get_override: Option<FnGetOverride>,
    set_override: Option<FnSetOverride>,
    get_autohide: Option<FnGetAutohide>,
    set_autohide: Option<FnSetAutohide>,
    menu_bar_exists: Option<FnMenuBarExists>,
    set_override_alpha: Option<FnSetOverrideAlpha>,
    reset_override_alphas: Option<FnResetOverrideAlphas>,
    transaction_create: Option<FnTransactionCreate>,
    transaction_set_max_reveal: Option<FnTransactionSetMaxReveal>,
    transaction_commit: Option<FnTransactionCommit>,
}

/// Resolve one symbol, printing whether it was found. Returns a transmuted
/// function pointer of the requested type, or None if `dlsym` returns null.
unsafe fn resolve<T>(handle: *mut c_void, name: &str) -> Option<T> {
    let c_name = CString::new(name).unwrap();
    let symbol = unsafe { libc::dlsym(handle, c_name.as_ptr()) };
    if symbol.is_null() {
        println!("  [MISSING] {name}");
        None
    } else {
        println!("  [ok]      {name}");
        // SAFETY: caller supplies the matching fn-pointer type T; size checked.
        assert_eq!(size_of::<T>(), size_of::<*mut c_void>());
        Some(unsafe { std::mem::transmute_copy::<*mut c_void, T>(&symbol) })
    }
}

impl SkyLightAdapter {
    fn load() -> Self {
        let handle = unsafe { libc::dlopen(SKYLIGHT_PATH.as_ptr().cast(), libc::RTLD_LAZY) };
        if handle.is_null() {
            panic!("dlopen SkyLight failed");
        }
        println!("resolved SkyLight symbols:");
        unsafe {
            SkyLightAdapter {
                handle,
                main_connection_id: resolve(handle, "SLSMainConnectionID"),
                get_override: resolve(handle, "SLSGetMenuBarVisibilityOverrideOnDisplay"),
                set_override: resolve(handle, "SLSSetMenuBarVisibilityOverrideOnDisplay"),
                get_autohide: resolve(handle, "SLSGetMenuBarAutohideEnabled"),
                set_autohide: resolve(handle, "SLSSetMenuBarAutohideEnabled"),
                menu_bar_exists: resolve(handle, "SLSMenuBarExists"),
                set_override_alpha: resolve(handle, "SLSSetMenuBarSystemOverrideAlpha"),
                reset_override_alphas: resolve(handle, "SLSResetMenuBarSystemOverrideAlphas"),
                transaction_create: resolve(handle, "SLSTransactionCreate"),
                transaction_set_max_reveal: resolve(
                    handle,
                    "SLSTransactionSetMenuBarOverrideMaximumReveal",
                ),
                transaction_commit: resolve(handle, "SLSTransactionCommit"),
            }
        }
    }

    fn cid(&self) -> i32 {
        unsafe { (self.main_connection_id.expect("no SLSMainConnectionID"))() }
    }

    fn read_override(&self, display: CGDirectDisplayID) -> Option<(i32, CGError)> {
        let get = self.get_override?;
        let mut value = 0i32;
        let err = unsafe { get(self.cid(), display, &mut value) };
        Some((value, err))
    }

    fn write_override(&self, display: CGDirectDisplayID, value: i32) -> CGError {
        let set = self
            .set_override
            .expect("no SLSSetMenuBarVisibilityOverrideOnDisplay");
        unsafe { set(self.cid(), display, value) }
    }

    fn read_autohide(&self) -> Option<(bool, CGError)> {
        let get = self.get_autohide?;
        let mut value = false;
        let err = unsafe { get(self.cid(), &mut value) };
        Some((value, err))
    }

    fn write_autohide(&self, value: bool) -> CGError {
        let set = self.set_autohide.expect("no SLSSetMenuBarAutohideEnabled");
        unsafe { set(self.cid(), value) }
    }

    fn write_override_alpha(&self, alpha: f64) -> CGError {
        let set = self
            .set_override_alpha
            .expect("no SLSSetMenuBarSystemOverrideAlpha");
        unsafe { set(self.cid(), alpha) }
    }

    fn reset_override_alpha(&self) -> CGError {
        let reset = self
            .reset_override_alphas
            .expect("no SLSResetMenuBarSystemOverrideAlphas");
        unsafe { reset(self.cid()) }
    }

    /// Create → set max-reveal → commit, as one transaction.
    fn set_max_reveal(&self, reveal: f64) -> CGError {
        let create = self.transaction_create.expect("no SLSTransactionCreate");
        let set = self
            .transaction_set_max_reveal
            .expect("no SLSTransactionSetMenuBarOverrideMaximumReveal");
        let commit = self.transaction_commit.expect("no SLSTransactionCommit");
        unsafe {
            let transaction = create(self.cid());
            set(transaction, reveal);
            commit(transaction, 0)
        }
    }
}

impl SkyLightPort for SkyLightAdapter {
    fn active_displays(&self) -> Vec<DisplayId> {
        active_displays()
    }

    fn visibility_override(&self, display: DisplayId) -> Result<i32, SkyLightError> {
        let (value, error) = self.read_override(display).ok_or(SkyLightError(-1))?;
        (error == 0).then_some(value).ok_or(SkyLightError(error))
    }

    fn set_visibility_override(&self, display: DisplayId, value: i32) -> Result<(), SkyLightError> {
        let error = self.write_override(display, value);
        (error == 0).then_some(()).ok_or(SkyLightError(error))
    }

    fn autohide(&self) -> Result<bool, SkyLightError> {
        let (enabled, error) = self.read_autohide().ok_or(SkyLightError(-1))?;
        (error == 0).then_some(enabled).ok_or(SkyLightError(error))
    }

    fn set_autohide(&self, enabled: bool) -> Result<(), SkyLightError> {
        let error = self.write_autohide(enabled);
        (error == 0).then_some(()).ok_or(SkyLightError(error))
    }

    fn set_override_alpha(&self, alpha: f64) -> Result<(), SkyLightError> {
        let error = self.write_override_alpha(alpha);
        (error == 0).then_some(()).ok_or(SkyLightError(error))
    }

    fn reset_override_alpha(&self) -> Result<(), SkyLightError> {
        let error = self.reset_override_alpha();
        (error == 0).then_some(()).ok_or(SkyLightError(error))
    }

    fn set_maximum_reveal(&self, reveal: f64) -> Result<(), SkyLightError> {
        let error = self.set_max_reveal(reveal);
        (error == 0).then_some(()).ok_or(SkyLightError(error))
    }
}

impl Drop for SkyLightAdapter {
    fn drop(&mut self) {
        unsafe { libc::dlclose(self.handle) };
    }
}

// ---------------------------------------------------------------------------
// Ctrl-C → restore hook
// ---------------------------------------------------------------------------

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

extern "C" fn on_sigint(_sig: i32) {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

fn install_sigint() {
    unsafe { libc::signal(libc::SIGINT, on_sigint as *const () as libc::sighandler_t) };
}

/// Hold for `secs`, printing a live one-line countdown, returning early on
/// Ctrl-C. Flushes each tick so the current phase is always obvious on screen.
fn hold(secs: f64) {
    use std::io::Write;
    let total = secs.ceil() as i64;
    for remaining in (1..=total).rev() {
        if INTERRUPTED.load(Ordering::SeqCst) {
            println!("\n   interrupted — restoring early");
            return;
        }
        print!("\r   ⏳ {remaining:>2}s … ");
        std::io::stdout().flush().ok();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    print!("\r              \r");
    std::io::stdout().flush().ok();
}

// ---------------------------------------------------------------------------
// Subcommands
// ---------------------------------------------------------------------------

fn cmd_probe(sky: &SkyLightAdapter) {
    println!("\nconnection id: {}", sky.cid());
    if let Some(exists) = sky.menu_bar_exists {
        println!("SLSMenuBarExists: {}", unsafe { exists(sky.cid()) });
    }
    match sky.read_autohide() {
        Some((value, err)) => println!("autohide enabled: {value} (AXErr {err})"),
        None => println!("autohide: symbol unavailable"),
    }
    println!("per-display visibility override:");
    for display in active_displays() {
        let main = if display == unsafe { CGMainDisplayID() } {
            " (main)"
        } else {
            ""
        };
        match sky.read_override(display) {
            Some((value, err)) => {
                println!("  display {display}{main}: override={value} (CGErr {err})")
            }
            None => println!("  display {display}{main}: get symbol unavailable"),
        }
    }
}

fn cmd_override(sky: &SkyLightAdapter, value: i32, secs: f64) {
    let displays = active_displays();
    // Snapshot current state so we can restore exactly.
    let previous: Vec<(CGDirectDisplayID, i32)> = displays
        .iter()
        .filter_map(|&display| sky.read_override(display).map(|(v, _)| (display, v)))
        .collect();

    println!(
        "setting visibility override = {value} on {} display(s) for {secs}s",
        displays.len()
    );
    for &display in &displays {
        let err = sky.write_override(display, value);
        println!("  display {display}: set override={value} → CGErr {err}");
    }

    hold(secs);

    println!("restoring previous override state:");
    for (display, restore) in previous {
        let err = sky.write_override(display, restore);
        println!("  display {display}: restore override={restore} → CGErr {err}");
    }
}

fn cmd_autohide(sky: &SkyLightAdapter, enable: bool, secs: f64) {
    let previous = sky.read_autohide().map(|(value, _)| value).unwrap_or(false);
    println!("setting autohide = {enable} for {secs}s (was {previous})");
    let err = sky.write_autohide(enable);
    println!("  set autohide={enable} → CGErr {err}");

    hold(secs);

    let err = sky.write_autohide(previous);
    println!("restore autohide={previous} → CGErr {err}");
}

/// Sweep the integer override enum on the main display, one value at a time
/// with a loud banner + countdown, so we can read off which value hides an
/// always-visible menu bar. Restores the original value at the end.
fn cmd_sweep(sky: &SkyLightAdapter, values: &[i32], phase_secs: f64) {
    let main = unsafe { CGMainDisplayID() };
    let start = sky.read_override(main).map(|(v, _)| v).unwrap_or(0);
    println!("starting override value on main display: {start}\n");

    for &value in values {
        if INTERRUPTED.load(Ordering::SeqCst) {
            break;
        }
        println!("╔══════════════════════════════════════════════╗");
        println!("║  OVERRIDE = {value}   ← watch the menu bar now      ",);
        println!("╚══════════════════════════════════════════════╝");
        let err = sky.write_override(main, value);
        println!("   set override={value} → CGErr {err}");
        hold(phase_secs);
        println!("   done value {value}\n");
    }

    let err = sky.write_override(main, start);
    println!("restored override={start} → CGErr {err}");
}

/// Force the whole menu bar to a given alpha (0 = fully transparent) for a
/// held interval, then clear the override. This is the candidate real
/// suppression lever for an always-visible bar.
fn cmd_alpha(sky: &SkyLightAdapter, alpha: f64, secs: f64) {
    println!("setting menu-bar system override alpha = {alpha} for {secs}s");
    let err = sky.write_override_alpha(alpha);
    println!("  set alpha={alpha} → CGErr {err}   ← watch the menu bar / hover the top edge");
    hold(secs);
    let err = sky.reset_override_alpha();
    println!("reset override alphas → CGErr {err}");
}

/// Re-apply an alpha override at ~60Hz for the interval. If the WindowServer
/// resets the alpha each frame, a one-shot won't stick but this will (possibly
/// with flicker), telling us the mechanism works and just needs persistence.
fn cmd_alpha_hold(sky: &SkyLightAdapter, alpha: f64, secs: f64) {
    println!("hammering menu-bar override alpha = {alpha} at ~60Hz for {secs}s");
    println!("  ← watch the menu bar / hover the top edge");
    let ticks = (secs * 60.0) as u64;
    for _ in 0..ticks {
        if INTERRUPTED.load(Ordering::SeqCst) {
            break;
        }
        sky.write_override_alpha(alpha);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    let err = sky.reset_override_alpha();
    println!("\nreset override alphas → CGErr {err}");
}

/// Your "infinite hover delay" idea, done as a reveal cap: enable autohide so
/// the bar is hidden, pin max-reveal to 0 so hovering the top edge reveals
/// nothing, hold while you test, then restore.
fn cmd_maxreveal(sky: &SkyLightAdapter, secs: f64) {
    let start_autohide = sky.read_autohide().map(|(v, _)| v).unwrap_or(false);
    println!("enabling autohide + pinning max reveal = 0 for {secs}s");
    println!("  (was autohide={start_autohide})");
    let err = sky.write_autohide(true);
    println!("  autohide=true → CGErr {err}");
    let err = sky.set_max_reveal(0.0);
    println!("  max reveal=0 → CGErr {err}   ← HOVER the top edge; bar should NOT appear");

    hold(secs);

    // Restore: lift the reveal cap and put autohide back where it was.
    let err = sky.set_max_reveal(10_000.0);
    println!("restore max reveal → CGErr {err}");
    let err = sky.write_autohide(start_autohide);
    println!("restore autohide={start_autohide} → CGErr {err}");
}

pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    install_sigint();
    let sky = SkyLightAdapter::load();

    let secs = |index: usize| args.get(index).and_then(|s| s.parse().ok()).unwrap_or(8.0);

    match args.first().map(String::as_str) {
        Some("probe") | None => cmd_probe(&sky),
        Some("sweep") => cmd_sweep(&sky, &[0, 1, 2, 3], secs(1)),
        Some("alpha") => match args.get(1).and_then(|s| s.parse::<f64>().ok()) {
            Some(alpha) => cmd_alpha(&sky, alpha, secs(2)),
            None => eprintln!("usage: exp-skylight alpha <0.0..1.0> [secs]"),
        },
        Some("alpha-hold") => match args.get(1).and_then(|s| s.parse::<f64>().ok()) {
            Some(alpha) => cmd_alpha_hold(&sky, alpha, secs(2)),
            None => eprintln!("usage: exp-skylight alpha-hold <0.0..1.0> [secs]"),
        },
        Some("maxreveal") => cmd_maxreveal(&sky, secs(1)),
        Some("override") => match args.get(1).and_then(|s| s.parse::<i32>().ok()) {
            Some(value) => cmd_override(&sky, value, secs(2)),
            None => eprintln!("usage: exp-skylight override <int-value> [secs]"),
        },
        Some("autohide") => match args.get(1).map(String::as_str) {
            Some("on") => cmd_autohide(&sky, true, secs(2)),
            Some("off") => cmd_autohide(&sky, false, secs(2)),
            _ => eprintln!("usage: exp-skylight autohide <on|off> [secs]"),
        },
        _ => eprintln!("usage: exp-skylight probe|override <hide|show>|autohide <on|off> [secs]"),
    }
}
