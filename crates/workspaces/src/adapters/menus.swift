import AppKit
import ApplicationServices
import Foundation

public typealias MenuCallback = @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void
public typealias SeparatorCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias ItemCallback = @convention(c) (
    UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafePointer<CChar>?, Bool, Bool
) -> Void

private func stringAttribute(_ element: AXUIElement, _ attribute: CFString) -> String? {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, attribute, &value) == .success else { return nil }
    return value as? String
}

private func boolAttribute(_ element: AXUIElement, _ attribute: CFString, fallback: Bool) -> Bool {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, attribute, &value) == .success else { return fallback }
    return (value as? Bool) ?? fallback
}

private func children(_ element: AXUIElement) -> [AXUIElement] {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &value) == .success,
          let array = value as? [AXUIElement]
    else { return [] }
    return array
}

private var permissionPrompted = false
private func menuBar(expectedName: String) -> AXUIElement? {
    if !permissionPrompted {
        permissionPrompted = true
        AXIsProcessTrustedWithOptions([kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary)
    }
    guard AXIsProcessTrusted(),
          let app = NSWorkspace.shared.frontmostApplication,
          expectedName.isEmpty || app.localizedName == expectedName
    else { return nil }
    let application = AXUIElementCreateApplication(app.processIdentifier)
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(application, kAXMenuBarAttribute as CFString, &value) == .success else {
        return nil
    }
    return (value as! AXUIElement)
}

private func shortcut(_ item: AXUIElement) -> String? {
    guard let character = stringAttribute(item, kAXMenuItemCmdCharAttribute as CFString),
          !character.isEmpty else { return nil }
    var value: CFTypeRef?
    var modifiers = 0
    if AXUIElementCopyAttributeValue(item, kAXMenuItemCmdModifiersAttribute as CFString, &value) == .success,
       let number = value as? NSNumber {
        modifiers = number.intValue
    }
    // AX modifier constants are C enum values not imported by Swift.
    let shift = 1 << 0
    let option = 1 << 1
    let control = 1 << 2
    let noCommand = 1 << 3
    var result = ""
    if modifiers & control != 0 { result += "⌃" }
    if modifiers & option != 0 { result += "⌥" }
    if modifiers & shift != 0 { result += "⇧" }
    if modifiers & noCommand == 0 { result += "⌘" }
    return result + character.uppercased()
}

private func withCString(_ value: String?, _ body: (UnsafePointer<CChar>?) -> Void) {
    guard let value else { body(nil); return }
    value.withCString(body)
}

@_cdecl("neosicht_copy_frontmost_menus")
public func neosichtCopyFrontmostMenus(
    _ appName: UnsafePointer<CChar>?,
    _ context: UnsafeMutableRawPointer?,
    _ menuCallback: MenuCallback,
    _ separatorCallback: SeparatorCallback,
    _ itemCallback: ItemCallback
) -> Bool {
    let expected = appName.map(String.init(cString:)) ?? ""
    guard let bar = menuBar(expectedName: expected) else { return false }
    for top in children(bar) {
        guard let title = stringAttribute(top, kAXTitleAttribute as CFString), !title.isEmpty else { continue }
        title.withCString { menuCallback(context, $0) }
        guard let menu = children(top).first else { continue }
        for item in children(menu) {
            let role = stringAttribute(item, kAXRoleAttribute as CFString)
            let label = stringAttribute(item, kAXTitleAttribute as CFString) ?? ""
            let separator = role == (kAXMenuItemRole as String) && label.isEmpty
            if separator { separatorCallback(context); continue }
            if label.isEmpty { continue }
            let marked = !(stringAttribute(item, kAXMenuItemMarkCharAttribute as CFString) ?? "").isEmpty
            let enabled = boolAttribute(item, kAXEnabledAttribute as CFString, fallback: true)
            label.withCString { labelPointer in
                withCString(shortcut(item)) { shortcutPointer in
                    itemCallback(context, labelPointer, shortcutPointer, marked, enabled)
                }
            }
        }
    }
    return true
}

@_cdecl("neosicht_activate_frontmost_menu_item")
public func neosichtActivateFrontmostMenuItem(
    _ appName: UnsafePointer<CChar>?,
    _ menuTitle: UnsafePointer<CChar>?,
    _ itemIndex: Int
) -> Bool {
    let expected = appName.map(String.init(cString:)) ?? ""
    let wanted = menuTitle.map(String.init(cString:)) ?? ""
    guard let bar = menuBar(expectedName: expected) else { return false }
    for top in children(bar) where stringAttribute(top, kAXTitleAttribute as CFString) == wanted {
        guard let menu = children(top).first else { return false }
        var entryIndex = 0
        for item in children(menu) {
            let role = stringAttribute(item, kAXRoleAttribute as CFString)
            let label = stringAttribute(item, kAXTitleAttribute as CFString) ?? ""
            let separator = role == (kAXMenuItemRole as String) && label.isEmpty
            if !separator && label.isEmpty { continue }
            if entryIndex == itemIndex {
                return !separator && AXUIElementPerformAction(item, kAXPressAction as CFString) == .success
            }
            entryIndex += 1
        }
    }
    return false
}
