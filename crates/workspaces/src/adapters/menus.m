#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>

typedef void (*neosicht_menu_cb)(void *, const char *);
typedef void (*neosicht_separator_cb)(void *);
typedef void (*neosicht_item_cb)(void *, const char *, const char *, bool, bool);

static NSString *stringAttribute(AXUIElementRef element, CFStringRef attribute) {
    CFTypeRef value = NULL;
    if (AXUIElementCopyAttributeValue(element, attribute, &value) != kAXErrorSuccess || value == NULL) return nil;
    if (CFGetTypeID(value) != CFStringGetTypeID()) { CFRelease(value); return nil; }
    return CFBridgingRelease(value);
}

static bool boolAttribute(AXUIElementRef element, CFStringRef attribute, bool fallback) {
    CFTypeRef value = NULL;
    if (AXUIElementCopyAttributeValue(element, attribute, &value) != kAXErrorSuccess || value == NULL) return fallback;
    bool result = CFGetTypeID(value) == CFBooleanGetTypeID() ? CFBooleanGetValue(value) : fallback;
    CFRelease(value);
    return result;
}

static NSArray *children(AXUIElementRef element) {
    CFTypeRef value = NULL;
    if (AXUIElementCopyAttributeValue(element, kAXChildrenAttribute, &value) != kAXErrorSuccess || value == NULL) return @[];
    if (CFGetTypeID(value) != CFArrayGetTypeID()) { CFRelease(value); return @[]; }
    return CFBridgingRelease(value);
}

static AXUIElementRef menuBarForApplication(const char *expectedName) {
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)@{
            (__bridge NSString *)kAXTrustedCheckOptionPrompt: @YES
        });
    });
    if (!AXIsProcessTrusted()) return NULL;
    NSRunningApplication *app = NSWorkspace.sharedWorkspace.frontmostApplication;
    if (app == nil) return NULL;
    NSString *expected = [NSString stringWithUTF8String:expectedName];
    if (expected.length > 0 && ![app.localizedName isEqualToString:expected]) return NULL;
    AXUIElementRef application = AXUIElementCreateApplication(app.processIdentifier);
    CFTypeRef menuBar = NULL;
    AXError error = AXUIElementCopyAttributeValue(application, kAXMenuBarAttribute, &menuBar);
    CFRelease(application);
    return error == kAXErrorSuccess ? menuBar : NULL;
}

static NSString *shortcutFor(AXUIElementRef item) {
    NSString *character = stringAttribute(item, kAXMenuItemCmdCharAttribute);
    if (character.length == 0) return nil;
    CFTypeRef modifiersValue = NULL;
    AXUIElementCopyAttributeValue(item, kAXMenuItemCmdModifiersAttribute, &modifiersValue);
    NSInteger modifiers = 0;
    if (modifiersValue != NULL && CFGetTypeID(modifiersValue) == CFNumberGetTypeID()) CFNumberGetValue(modifiersValue, kCFNumberNSIntegerType, &modifiers);
    if (modifiersValue != NULL) CFRelease(modifiersValue);
    NSMutableString *shortcut = [NSMutableString string];
    if (modifiers & kAXMenuItemModifierControl) [shortcut appendString:@"⌃"];
    if (modifiers & kAXMenuItemModifierOption) [shortcut appendString:@"⌥"];
    if (modifiers & kAXMenuItemModifierShift) [shortcut appendString:@"⇧"];
    if (!(modifiers & kAXMenuItemModifierNoCommand)) [shortcut appendString:@"⌘"];
    [shortcut appendString:character.uppercaseString];
    return shortcut;
}

bool neosicht_copy_frontmost_menus(const char *appName, void *context, neosicht_menu_cb menuCallback, neosicht_separator_cb separatorCallback, neosicht_item_cb itemCallback) {
    @autoreleasepool {
        AXUIElementRef menuBar = menuBarForApplication(appName);
        if (menuBar == NULL) return false;
        for (id topObject in children(menuBar)) {
            AXUIElementRef top = (__bridge AXUIElementRef)topObject;
            NSString *title = stringAttribute(top, kAXTitleAttribute);
            if (title.length == 0) continue;
            menuCallback(context, title.UTF8String);
            NSArray *topChildren = children(top);
            if (topChildren.count == 0) continue;
            AXUIElementRef menu = (__bridge AXUIElementRef)topChildren.firstObject;
            for (id itemObject in children(menu)) {
                AXUIElementRef item = (__bridge AXUIElementRef)itemObject;
                NSString *role = stringAttribute(item, kAXRoleAttribute);
                NSString *label = stringAttribute(item, kAXTitleAttribute);
                if ([role isEqualToString:(__bridge NSString *)kAXMenuItemRole] && label.length == 0) {
                    separatorCallback(context);
                    continue;
                }
                if (label.length == 0) continue;
                NSString *shortcut = shortcutFor(item);
                CFTypeRef markValue = NULL;
                bool checked = false;
                if (AXUIElementCopyAttributeValue(item, kAXMenuItemMarkCharAttribute, &markValue) == kAXErrorSuccess && markValue != NULL) {
                    checked = CFGetTypeID(markValue) == CFStringGetTypeID() && [(__bridge NSString *)markValue length] > 0;
                    CFRelease(markValue);
                }
                itemCallback(context, label.UTF8String, shortcut.UTF8String, checked, boolAttribute(item, kAXEnabledAttribute, true));
            }
        }
        CFRelease(menuBar);
        return true;
    }
}

bool neosicht_activate_frontmost_menu_item(const char *appName, const char *menuTitle, size_t itemIndex) {
    @autoreleasepool {
        AXUIElementRef menuBar = menuBarForApplication(appName);
        if (menuBar == NULL) return false;
        NSString *wantedMenu = [NSString stringWithUTF8String:menuTitle];
        bool activated = false;
        for (id topObject in children(menuBar)) {
            AXUIElementRef top = (__bridge AXUIElementRef)topObject;
            if (![stringAttribute(top, kAXTitleAttribute) isEqualToString:wantedMenu]) continue;
            NSArray *topChildren = children(top);
            if (topChildren.count == 0) break;
            AXUIElementRef menu = (__bridge AXUIElementRef)topChildren.firstObject;
            size_t entryIndex = 0;
            for (id itemObject in children(menu)) {
                AXUIElementRef item = (__bridge AXUIElementRef)itemObject;
                NSString *role = stringAttribute(item, kAXRoleAttribute);
                NSString *label = stringAttribute(item, kAXTitleAttribute);
                bool isSeparator = [role isEqualToString:(__bridge NSString *)kAXMenuItemRole]
                    && label.length == 0;
                if (!isSeparator && label.length == 0) continue;
                if (entryIndex == itemIndex) {
                    activated = !isSeparator
                        && AXUIElementPerformAction(item, kAXPressAction) == kAXErrorSuccess;
                    break;
                }
                entryIndex += 1;
            }
            break;
        }
        CFRelease(menuBar);
        return activated;
    }
}
