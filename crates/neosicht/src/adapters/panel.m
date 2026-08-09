// AppKit adapter for positioning and input routing of the persistent shell
// panel. Rust sees only the plain C ABI below and never touches Obj-C objects.

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

@interface NeosichtInputPassthrough : NSObject
@property(nonatomic, weak) NSWindow *window;
@property(nonatomic) BOOL extended;
@property(nonatomic) double barHeight;
@property(nonatomic, strong) id globalMouseMonitor;
@property(nonatomic, strong) id localMouseMonitor;
- (void)update;
@end

@implementation NeosichtInputPassthrough
- (void)update {
    NSWindow *window = self.window;
    if (window == nil) {
        return;
    }

    BOOL acceptsMouse = self.extended;
    if (!acceptsMouse) {
        NSRect frame = window.frame;
        NSPoint cursor = [NSEvent mouseLocation];
        acceptsMouse = NSPointInRect(cursor, frame)
            && cursor.y >= NSMaxY(frame) - self.barHeight;
    }
    window.ignoresMouseEvents = !acceptsMouse;
}

- (void)dealloc {
    if (self.globalMouseMonitor != nil) {
        [NSEvent removeMonitor:self.globalMouseMonitor];
    }
    if (self.localMouseMonitor != nil) {
        [NSEvent removeMonitor:self.localMouseMonitor];
    }
}
@end

@interface NeosichtPanelState : NSObject
@property(nonatomic, weak) NSWindow *window;
@end

@implementation NeosichtPanelState
@end

static char const neosicht_input_passthrough_key = 0;
static char const neosicht_panel_state_key = 0;

static NeosichtPanelState *neosicht_panel_state(void) {
    NSApplication *app = [NSApplication sharedApplication];
    NeosichtPanelState *state =
        objc_getAssociatedObject(app, &neosicht_panel_state_key);
    if (state == nil) {
        state = [[NeosichtPanelState alloc] init];
        objc_setAssociatedObject(app, &neosicht_panel_state_key,
                                 state, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
    }
    return state;
}

static NeosichtInputPassthrough *neosicht_input_passthrough(NSWindow *window) {
    NeosichtInputPassthrough *passthrough =
        objc_getAssociatedObject(window, &neosicht_input_passthrough_key);
    if (passthrough != nil) {
        return passthrough;
    }

    passthrough = [[NeosichtInputPassthrough alloc] init];
    passthrough.window = window;
    __weak NeosichtInputPassthrough *weakPassthrough = passthrough;
    NSEventMask movement = NSEventMaskMouseMoved
        | NSEventMaskLeftMouseDragged
        | NSEventMaskRightMouseDragged
        | NSEventMaskOtherMouseDragged;
    passthrough.globalMouseMonitor =
        [NSEvent addGlobalMonitorForEventsMatchingMask:movement
                                               handler:^(__unused NSEvent *event) {
            [weakPassthrough update];
        }];
    passthrough.localMouseMonitor =
        [NSEvent addLocalMonitorForEventsMatchingMask:movement
                                              handler:^NSEvent *(NSEvent *event) {
            [weakPassthrough update];
            return event;
        }];
    objc_setAssociatedObject(window, &neosicht_input_passthrough_key,
                             passthrough, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
    return passthrough;
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

BOOL neosicht_set_panel_interaction(BOOL extended, double bar_height) {
    NSWindow *window = neosicht_panel_state().window;
    if (window == nil) {
        return NO;
    }

    NeosichtInputPassthrough *passthrough =
        neosicht_input_passthrough(window);
    passthrough.extended = extended;
    if (!extended) {
        passthrough.barHeight = bar_height;
    }
    [passthrough update];
    return YES;
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

    // Retain the identity of the panel this adapter owns; later interaction
    // changes must never guess using the application's window ordering.
    neosicht_panel_state().window = w;

    // Remove the menu-bar/notch clamp + refuse key focus, then place the frame.
    neosicht_prepare_shell_window_class();

    // Defer to the next runloop turn: pinning is called from inside gpui's
    // effect cycle, and a synchronous setFrame: re-enters gpui's draw/resize
    // handling while its state is borrowed ("RefCell already borrowed"),
    // leaving gpui's viewport size stale and popovers clamped to the old
    // bar-height window.
    dispatch_async(dispatch_get_main_queue(), ^{
        [w setLevel:level];
        [w setHasShadow:NO];
        [w setFrame:target display:NO];
    });

    // The frame is applied asynchronously; report the requested offset.
    return top;
}
