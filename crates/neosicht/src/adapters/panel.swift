import AppKit
import CoreGraphics
import Foundation
import ObjectiveC.runtime

private final class InputPassthrough {
    weak var window: NSWindow?
    var extended = false
    var barHeight = 0.0
    var globalMonitor: Any?
    var localMonitor: Any?

    init(window: NSWindow) {
        self.window = window
        let mask: NSEvent.EventTypeMask = [.mouseMoved, .leftMouseDragged, .rightMouseDragged, .otherMouseDragged]
        globalMonitor = NSEvent.addGlobalMonitorForEvents(matching: mask) { [weak self] _ in self?.update() }
        localMonitor = NSEvent.addLocalMonitorForEvents(matching: mask) { [weak self] event in
            self?.update()
            return event
        }
    }

    deinit {
        if let globalMonitor { NSEvent.removeMonitor(globalMonitor) }
        if let localMonitor { NSEvent.removeMonitor(localMonitor) }
    }

    func update() {
        guard let window else { return }
        var acceptsMouse = extended
        if !acceptsMouse {
            let cursor = NSEvent.mouseLocation
            let frame = window.frame
            acceptsMouse = frame.contains(cursor) && cursor.y >= frame.maxY - barHeight
        }
        window.ignoresMouseEvents = !acceptsMouse
    }
}

private final class PanelState {
    static let shared = PanelState()
    weak var window: NSWindow?
    var passthrough: InputPassthrough?
}

private typealias ConstrainImplementation = @convention(c) (
    AnyObject, Selector, NSRect, NSScreen?
) -> NSRect
private let unconstrained: ConstrainImplementation = { _, _, frame, _ in frame }
private typealias BooleanImplementation = @convention(c) (AnyObject, Selector) -> Bool
private let returnsNo: BooleanImplementation = { _, _ in false }

private func replaceMethod(_ cls: AnyClass, _ selector: Selector, _ implementation: UnsafeRawPointer) {
    guard let method = class_getInstanceMethod(cls, selector) else { return }
    class_replaceMethod(cls, selector, unsafeBitCast(implementation, to: IMP.self), method_getTypeEncoding(method))
}

private func prepareShellWindowClass() {
    let cls: AnyClass = NSClassFromString("GPUIPanel") ?? NSPanel.self
    replaceMethod(
        cls,
        NSSelectorFromString("constrainFrameRect:toScreen:"),
        unsafeBitCast(unconstrained, to: UnsafeRawPointer.self)
    )
    replaceMethod(
        cls,
        NSSelectorFromString("canBecomeKeyWindow"),
        unsafeBitCast(returnsNo, to: UnsafeRawPointer.self)
    )
    replaceMethod(
        cls,
        NSSelectorFromString("canBecomeMainWindow"),
        unsafeBitCast(returnsNo, to: UnsafeRawPointer.self)
    )
}

@_cdecl("neosicht_set_panel_interaction")
public func neosichtSetPanelInteraction(_ extended: Int8, _ barHeight: Double) -> Int8 {
    guard let window = PanelState.shared.window else { return 0 }
    let passthrough: InputPassthrough
    if let existing = PanelState.shared.passthrough {
        passthrough = existing
    } else {
        passthrough = InputPassthrough(window: window)
        PanelState.shared.passthrough = passthrough
    }
    passthrough.extended = extended != 0
    if extended == 0 { passthrough.barHeight = barHeight }
    passthrough.update()
    return 1
}

@_cdecl("neosicht_menu_bar_height")
public func neosichtMenuBarHeight() -> Double {
    guard let screen = NSScreen.main else { return 0 }
    return screen.frame.height - (screen.visibleFrame.origin.y + screen.visibleFrame.height)
}

@_cdecl("neosicht_pin_shell_window")
public func neosichtPinShellWindow(
    _ levelKey: Int32,
    _ x: Double,
    _ top: Double,
    _ width: Double,
    _ height: Double
) -> Double {
    guard let window = NSApplication.shared.windows.first,
          let key = CGWindowLevelKey(rawValue: levelKey)
    else { return -1 }
    let screen = window.screen ?? NSScreen.main
    guard let screen else { return -1 }
    let frame = screen.frame
    let target = NSRect(
        x: frame.origin.x + x,
        y: frame.origin.y + frame.height - top - height,
        width: width,
        height: height
    )
    PanelState.shared.window = window
    prepareShellWindowClass()
    let level = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(key)))
    DispatchQueue.main.async {
        window.level = level
        window.hasShadow = false
        window.setFrame(target, display: false)
    }
    return top
}
