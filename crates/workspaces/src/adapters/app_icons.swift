import AppKit
import Foundation

@_cdecl("workspaces_copy_app_icon_png")
public func workspacesCopyAppIconPNG(
    _ bundleID: UnsafePointer<CChar>?,
    _ appName: UnsafePointer<CChar>?,
    _ pixelSize: Int32,
    _ outBytes: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>?
) -> Int64 {
    let bundle = bundleID.flatMap { $0.pointee == 0 ? nil : String(cString: $0) }
    let name = appName.flatMap { $0.pointee == 0 ? nil : String(cString: $0) }
    var app = bundle.flatMap { NSRunningApplication.runningApplications(withBundleIdentifier: $0).first }
    if app == nil, let name {
        app = NSWorkspace.shared.runningApplications.first { $0.localizedName == name }
    }
    var icon = app?.bundleURL.map { NSWorkspace.shared.icon(forFile: $0.path) }
    if icon == nil, let bundle, let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundle) {
        icon = NSWorkspace.shared.icon(forFile: url.path)
    }
    guard let icon else { return 0 }
    var rect = NSRect(x: 0, y: 0, width: Int(pixelSize), height: Int(pixelSize))
    guard let image = icon.cgImage(forProposedRect: &rect, context: nil, hints: nil),
          let png = NSBitmapImageRep(cgImage: image).representation(using: .png, properties: [:])
    else { return 0 }
    let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: png.count)
    png.copyBytes(to: buffer, count: png.count)
    outBytes?.pointee = buffer
    return Int64(png.count)
}

@_cdecl("workspaces_free_icon_png")
public func workspacesFreeIconPNG(_ bytes: UnsafeMutablePointer<UInt8>?) {
    bytes?.deallocate()
}
