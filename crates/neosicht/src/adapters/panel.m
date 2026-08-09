// Experiment 0/2 — position the shell panel at the very top of the screen.
//
// gpui/AppKit clamps a bar-height window below the menu bar (y=32 on a notched
// display). Two facts let us defeat that with public API only, à la Barik:
//   1. A window whose frame equals the full screen cannot be constrained
//      downward (nowhere to go), so it lands at y=0.
//   2. A low window level (below normal app windows) means the full-screen
//      transparent panel does not intercept clicks meant for apps.
//
// This exports one plain C function; Rust never touches the Obj-C objects.

#import <AppKit/AppKit.h>
#import <CoreGraphics/CoreGraphics.h>
#import <objc/runtime.h>

// AppKit's -[NSWindow constrainFrameRect:toScreen:] keeps windows out of the
// menu-bar / notch strip, which is why our bar clamps to y=32 on a notched
// display. Override it on gpui's window class to return the frame unchanged.
// This is a runtime method replacement scoped to OUR process's own class only
// — no injection into other processes, SIP-safe.
static NSRect neosicht_unconstrained(id self, SEL _cmd, NSRect frame, NSScreen *screen) {
    (void)self;
    (void)_cmd;
    (void)screen;
    return frame;
}

// gpui's GPUIPanel returns canBecomeKeyWindow = YES, so clicking the bar makes
// it the key window and the previously-focused app's text field resigns key
// (the cursor drops) even though the app stays frontmost. A shell bar must
// never take key focus — like the real menu bar / Dock, it should receive
// clicks without becoming key. Force both to NO.
static BOOL neosicht_returns_no(id self, SEL _cmd) {
    (void)self;
    (void)_cmd;
    return NO;
}

static void neosicht_prepare_shell_window_class(void) {
    Class cls = objc_getClass("GPUIPanel");
    if (cls == NULL) {
        cls = [NSPanel class];
    }

    SEL constrain = @selector(constrainFrameRect:toScreen:);
    Method existing = class_getInstanceMethod(cls, constrain);
    if (existing != NULL) {
        class_replaceMethod(cls, constrain, (IMP)neosicht_unconstrained,
                            method_getTypeEncoding(existing));
    }

    SEL can_key = @selector(canBecomeKeyWindow);
    Method key_method = class_getInstanceMethod(cls, can_key);
    if (key_method != NULL) {
        class_replaceMethod(cls, can_key, (IMP)neosicht_returns_no,
                            method_getTypeEncoding(key_method));
    }

    SEL can_main = @selector(canBecomeMainWindow);
    Method main_method = class_getInstanceMethod(cls, can_main);
    if (main_method != NULL) {
        class_replaceMethod(cls, can_main, (IMP)neosicht_returns_no,
                            method_getTypeEncoding(main_method));
    }
}

// The exact menu-bar / notch band height on the main screen: the top inset
// between the full frame and the visible frame. Sizing the bar to this makes it
// cover the native menu bar exactly without spilling below the notch.
double neosicht_menu_bar_height(void) {
    NSScreen *screen = [NSScreen mainScreen];
    NSRect f = screen.frame;
    NSRect v = screen.visibleFrame;
    return f.size.height - (v.origin.y + v.size.height);
}

// Pin the process's shell window to an explicit rect on the main screen, given
// in top-left screen coordinates (`x`, `top` = distance from the screen's top
// edge, `width`, `height`), at the window level for `cg_level_key` (a
// CGWindowLevelKey). A level above the menu bar plus the constrain override
// lets the window sit inside the notch band. Returns the resulting top-edge
// offset from the screen top, or -1 if no window.
double neosicht_pin_shell_window(int cg_level_key, double x, double top,
                                 double width, double height) {
    NSApplication *app = [NSApplication sharedApplication];
    NSArray<NSWindow *> *windows = [app windows];
    if (windows.count == 0) {
        return -1;
    }
    CGWindowLevel level = CGWindowLevelForKey((CGWindowLevelKey)cg_level_key);
    NSWindow *w = windows.firstObject;
    NSScreen *screen = w.screen ?: [NSScreen mainScreen];
    NSRect sf = screen.frame;

    // Convert top-left screen coords to Cocoa's bottom-left window frame.
    NSRect target = NSMakeRect(sf.origin.x + x,
                               sf.origin.y + sf.size.height - top - height,
                               width, height);

    // Remove the menu-bar/notch clamp + refuse key focus, then place the frame.
    neosicht_prepare_shell_window_class();
    [w setLevel:level];
    [w setFrame:target display:YES];

    NSRect wf = w.frame;
    double window_top = wf.origin.y + wf.size.height;
    double screen_top = sf.origin.y + sf.size.height;
    return screen_top - window_top;
}
