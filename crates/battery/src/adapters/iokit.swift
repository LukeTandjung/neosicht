import Foundation
import IOKit.ps

@_cdecl("neosicht_read_battery")
public func neosichtReadBattery(
    _ currentCapacity: UnsafeMutablePointer<Int32>?,
    _ maximumCapacity: UnsafeMutablePointer<Int32>?,
    _ charging: UnsafeMutablePointer<Bool>?,
    _ pluggedIn: UnsafeMutablePointer<Bool>?
) -> Int32 {
    guard let snapshot = IOPSCopyPowerSourcesInfo()?.takeRetainedValue(),
          let sourceList = IOPSCopyPowerSourcesList(snapshot)?.takeRetainedValue() as? [AnyObject]
    else { return -1 }

    for source in sourceList {
        guard let description = IOPSGetPowerSourceDescription(snapshot, source)?.takeUnretainedValue()
                as? NSDictionary,
              let current = description.object(forKey: kIOPSCurrentCapacityKey) as? NSNumber,
              let maximum = description.object(forKey: kIOPSMaxCapacityKey) as? NSNumber,
              let isCharging = description.object(forKey: kIOPSIsChargingKey) as? NSNumber,
              let state = description.object(forKey: kIOPSPowerSourceStateKey) as? String
        else { continue }

        currentCapacity?.pointee = current.int32Value
        maximumCapacity?.pointee = maximum.int32Value
        charging?.pointee = isCharging.boolValue
        pluggedIn?.pointee = state == kIOPSACPowerValue
        return 1
    }
    return 0
}
