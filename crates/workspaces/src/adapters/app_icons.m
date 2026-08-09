// Application icon lookup, barik-style: resolve the running application by
// bundle identifier (preferred) or localized name, then encode NSWorkspace's
// icon for its bundle as PNG. Exports plain C functions; Rust never touches
// the Obj-C objects.

#import <AppKit/AppKit.h>

// PNG bytes of the icon for the application identified by `bundle_id`
// (preferred) or `app_name`, rendered at `pixel_size`. On success writes a
// malloc'd buffer to *out_bytes — free it with workspaces_free_icon_png — and
// returns its length. Returns 0 when the application or icon cannot be
// resolved.
int64_t workspaces_copy_app_icon_png(const char *bundle_id, const char *app_name,
                                     int32_t pixel_size, uint8_t **out_bytes) {
    @autoreleasepool {
        NSString *bundle = (bundle_id && bundle_id[0])
                               ? [NSString stringWithUTF8String:bundle_id]
                               : nil;
        NSString *name = (app_name && app_name[0])
                             ? [NSString stringWithUTF8String:app_name]
                             : nil;

        NSRunningApplication *app = nil;
        if (bundle) {
            app = [NSRunningApplication runningApplicationsWithBundleIdentifier:bundle]
                      .firstObject;
        }
        if (app == nil && name) {
            for (NSRunningApplication *candidate in
                 [NSWorkspace sharedWorkspace].runningApplications) {
                if ([candidate.localizedName isEqualToString:name]) {
                    app = candidate;
                    break;
                }
            }
        }

        NSImage *icon = nil;
        if (app.bundleURL != nil) {
            icon = [[NSWorkspace sharedWorkspace] iconForFile:app.bundleURL.path];
        }
        if (icon == nil && bundle) {
            NSURL *url = [[NSWorkspace sharedWorkspace]
                URLForApplicationWithBundleIdentifier:bundle];
            if (url != nil) {
                icon = [[NSWorkspace sharedWorkspace] iconForFile:url.path];
            }
        }
        if (icon == nil) {
            return 0;
        }

        NSRect rect = NSMakeRect(0, 0, pixel_size, pixel_size);
        CGImageRef cg = [icon CGImageForProposedRect:&rect
                                             context:nil
                                               hints:nil];
        if (cg == NULL) {
            return 0;
        }
        NSBitmapImageRep *rep = [[NSBitmapImageRep alloc] initWithCGImage:cg];
        NSData *png = [rep representationUsingType:NSBitmapImageFileTypePNG
                                        properties:@{}];
        if (png == nil || png.length == 0) {
            return 0;
        }

        uint8_t *buffer = malloc(png.length);
        if (buffer == NULL) {
            return 0;
        }
        memcpy(buffer, png.bytes, png.length);
        *out_bytes = buffer;
        return (int64_t)png.length;
    }
}

void workspaces_free_icon_png(uint8_t *bytes) {
    free(bytes);
}
