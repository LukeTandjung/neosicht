import AppKit
import EventKit
import Foundation

@_cdecl("neosicht_calendar_events")
public func neosichtCalendarEvents(
    _ startsAt: Double,
    _ endsBefore: Double,
    _ errorCode: UnsafeMutablePointer<Int32>?
) -> UnsafeMutablePointer<CChar>? {
    errorCode?.pointee = 0
    var authorization = EKEventStore.authorizationStatus(for: .event)
    if authorization == .notDetermined {
        let semaphore = DispatchSemaphore(value: 0)
        var granted = false
        EKEventStore().requestFullAccessToEvents { allowed, error in
            granted = allowed && error == nil
            semaphore.signal()
        }
        guard semaphore.wait(timeout: .now() + 30) == .success, granted else {
            errorCode?.pointee = 1
            return nil
        }
        authorization = EKEventStore.authorizationStatus(for: .event)
    }
    guard authorization == .fullAccess else {
        errorCode?.pointee = 1
        return nil
    }

    let store = EKEventStore()
    let predicate = store.predicateForEvents(
        withStart: Date(timeIntervalSince1970: startsAt),
        end: Date(timeIntervalSince1970: endsBefore),
        calendars: nil
    )
    let encoded: [[String: Any]] = store.events(matching: predicate).map { event in
        var entry: [String: Any] = [
            "title": event.title ?? "Untitled",
            "starts_at": event.startDate.timeIntervalSince1970,
            "all_day": event.isAllDay,
            "calendar_name": event.calendar.title,
        ]
        if let color = NSColor(cgColor: event.calendar.cgColor)?.usingColorSpace(.sRGB) {
            let red = Int((color.redComponent * 255).rounded())
            let green = Int((color.greenComponent * 255).rounded())
            let blue = Int((color.blueComponent * 255).rounded())
            entry["calendar_color"] = (red << 16) | (green << 8) | blue
        }
        return entry
    }
    guard let json = try? JSONSerialization.data(withJSONObject: encoded),
          let text = String(data: json, encoding: .utf8)
    else {
        errorCode?.pointee = 3
        return nil
    }
    return strdup(text)
}

@_cdecl("neosicht_free_string")
public func neosichtFreeString(_ value: UnsafeMutablePointer<CChar>?) {
    free(value)
}
