import CoreLocation
import Foundation

private final class LocationRequest: NSObject, CLLocationManagerDelegate {
    let semaphore = DispatchSemaphore(value: 0)
    var manager: CLLocationManager?
    var geocoder: CLGeocoder?
    var json: String?
    var errorCode: Int32 = 0
    private var finished = false

    func finish(_ code: Int32) {
        guard !finished else { return }
        finished = true
        errorCode = code
        semaphore.signal()
    }

    func begin() {
        let manager = CLLocationManager()
        self.manager = manager
        manager.delegate = self
        switch manager.authorizationStatus {
        case .notDetermined:
            manager.requestWhenInUseAuthorization()
        case .authorized, .authorizedAlways:
            if let cached = manager.location,
               abs(cached.timestamp.timeIntervalSinceNow) < 3600 {
                locationManager(manager, didUpdateLocations: [cached])
            } else {
                manager.requestLocation()
            }
        default:
            finish(1)
        }
    }

    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        switch manager.authorizationStatus {
        case .authorized, .authorizedAlways: manager.requestLocation()
        case .notDetermined: break
        default: finish(1)
        }
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard let location = locations.last else {
            finish(2)
            return
        }
        let geocoder = CLGeocoder()
        self.geocoder = geocoder
        geocoder.reverseGeocodeLocation(location) { [self] placemarks, _ in
            let placemark = placemarks?.first
            let place = placemark?.locality ?? placemark?.subLocality
                ?? placemark?.administrativeArea ?? "Current location"
            let value: [String: Any] = [
                "latitude": location.coordinate.latitude,
                "longitude": location.coordinate.longitude,
                "place_name": place,
            ]
            if let data = try? JSONSerialization.data(withJSONObject: value),
               let text = String(data: data, encoding: .utf8) {
                json = text
                finish(0)
            } else {
                finish(3)
            }
        }
    }

    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        finish(2)
    }
}

@_cdecl("neosicht_current_location")
public func neosichtCurrentLocation(
    _ errorCode: UnsafeMutablePointer<Int32>?
) -> UnsafeMutablePointer<CChar>? {
    let request = LocationRequest()
    DispatchQueue.main.async { request.begin() }
    guard request.semaphore.wait(timeout: .now() + 120) == .success else {
        errorCode?.pointee = 2
        return nil
    }
    errorCode?.pointee = request.errorCode
    guard let json = request.json else { return nil }
    return strdup(json)
}

@_cdecl("neosicht_weather_free_string")
public func neosichtWeatherFreeString(_ value: UnsafeMutablePointer<CChar>?) {
    free(value)
}
