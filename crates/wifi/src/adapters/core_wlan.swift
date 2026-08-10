import CoreWLAN
import Darwin
import Foundation
import SecurityFoundation

private func wifiInterface() -> CWInterface? {
    CWWiFiClient.shared().interface()
}

@_cdecl("neosicht_wifi_snapshot")
public func neosichtWifiSnapshot() -> UnsafeMutablePointer<CChar>? {
    guard let interface = wifiInterface() else { return nil }
    var rows: [[String: Any]] = []
    let knownSSIDs = Set((interface.configuration()?.networkProfiles.array as? [CWNetworkProfile] ?? [])
        .compactMap { profile in
            profile.ssidData.flatMap { String(data: $0, encoding: .utf8) }
        })
    if interface.powerOn() {
        do {
            let scanned = try interface.scanForNetworks(withSSID: nil)
            var strongest: [String: CWNetwork] = [:]
            for network in scanned {
                guard let ssid = network.ssid, !ssid.isEmpty else { continue }
                if strongest[ssid] == nil || network.rssiValue > strongest[ssid]!.rssiValue {
                    strongest[ssid] = network
                }
            }
            rows = strongest.values
                .sorted { $0.rssiValue > $1.rssiValue }
                .map { network in
                    [
                        "ssid": network.ssid ?? "",
                        "signal": network.rssiValue,
                        "secure": !network.supportsSecurity(.none),
                        "known": knownSSIDs.contains(network.ssid ?? ""),
                    ]
                }
        } catch {
            NSLog("Neosicht CoreWLAN scan failed: %@", String(describing: error))
            rows = []
        }
    }
    let payload: [String: Any] = [
        "enabled": interface.powerOn(),
        "connected": interface.ssid() ?? NSNull(),
        "networks": rows,
    ]
    guard let data = try? JSONSerialization.data(withJSONObject: payload),
          let string = String(data: data, encoding: .utf8)
    else { return nil }
    return strdup(string)
}

@_cdecl("neosicht_wifi_set_enabled")
public func neosichtWifiSetEnabled(_ enabled: Int32) -> Int32 {
    guard let interface = wifiInterface() else { return 0 }
    do {
        try interface.setPower(enabled != 0)
        return 1
    } catch {
        return 0
    }
}

@_cdecl("neosicht_wifi_join")
public func neosichtWifiJoin(
    _ ssidPointer: UnsafePointer<CChar>?,
    _ passwordPointer: UnsafePointer<CChar>?,
    _ remember: Int32
) -> Int32 {
    guard let interface = wifiInterface(), let ssidPointer else { return 0 }
    let ssid = String(cString: ssidPointer)
    do {
        let networks = try interface.scanForNetworks(withName: ssid)
        guard let network = networks.max(by: { $0.rssiValue < $1.rssiValue }) else { return 0 }
        let password = passwordPointer.map(String.init(cString:))
        try interface.associate(to: network, password: password)
        if remember == 0, let current = interface.configuration() {
            let updated = CWMutableConfiguration(configuration: current)
            let profiles = current.networkProfiles.array.compactMap { $0 as? CWNetworkProfile }
            updated.networkProfiles = NSOrderedSet(array: profiles.filter { profile in
                guard let data = profile.ssidData,
                      let profileSSID = String(data: data, encoding: .utf8)
                else { return true }
                return profileSSID != ssid
            })
            try interface.commitConfiguration(updated, authorization: nil)
        }
        return 1
    } catch {
        return 0
    }
}
