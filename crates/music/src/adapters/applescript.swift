import Foundation

private func runScript(_ source: String, errorCode: UnsafeMutablePointer<Int32>?) -> NSAppleEventDescriptor? {
    var error: NSDictionary?
    let result = NSAppleScript(source: source)?.executeAndReturnError(&error)
    if let number = error?[NSAppleScript.errorNumber] as? NSNumber {
        errorCode?.pointee = number.intValue == -1743 ? 1 : 2
        return nil
    }
    return result
}

private func readPlayer(
    _ application: String,
    includeArtwork: Bool,
    errorCode: UnsafeMutablePointer<Int32>?
) -> [String: Any]? {
    let spotify = application == "Spotify"
    let artwork = includeArtwork
        ? (spotify ? "artwork url of currentTrack" : "(get data of artwork 1 of currentTrack)")
        : "\"\""
    let source = """
    if application "\(application)" is running then
    tell application "\(application)"
    if player state is playing or player state is paused then
    set currentTrack to current track
    set stateText to "paused"
    if player state is playing then set stateText to "playing"
    set artworkValue to ""
    try
    set artworkValue to \(artwork)
    on error
    set artworkValue to ""
    end try
    return {stateText, name of currentTrack, artist of currentTrack, artworkValue, player position, duration of currentTrack}
    end if
    end tell
    end if
    return ""
    """
    guard let result = runScript(source, errorCode: errorCode),
          result.descriptorType == typeAEList,
          result.numberOfItems == 6,
          let state = result.atIndex(1)?.stringValue,
          let title = result.atIndex(2)?.stringValue,
          let artist = result.atIndex(3)?.stringValue
    else { return nil }

    let artworkDescriptor = result.atIndex(4)
    let artworkURL = spotify ? artworkDescriptor?.stringValue ?? "" : ""
    var artworkData = ""
    if !spotify, includeArtwork, let data = artworkDescriptor?.data, !data.isEmpty {
        artworkData = data.base64EncodedString()
    }
    var duration = result.atIndex(6)?.doubleValue ?? 0
    if spotify { duration /= 1000 }
    return [
        "application": spotify ? "spotify" : "apple_music",
        "state": state,
        "title": title,
        "artist": artist,
        "artwork_url": artworkURL,
        "artwork_data": artworkData,
        "position_seconds": result.atIndex(5)?.doubleValue ?? 0,
        "duration_seconds": duration,
    ]
}

@_cdecl("neosicht_now_playing")
public func neosichtNowPlaying(
    _ includeArtwork: Int32,
    _ errorCode: UnsafeMutablePointer<Int32>?
) -> UnsafeMutablePointer<CChar>? {
    errorCode?.pointee = 0
    let spotify = readPlayer("Spotify", includeArtwork: includeArtwork != 0, errorCode: errorCode)
    if errorCode?.pointee != 0 { return nil }
    let music = readPlayer("Music", includeArtwork: includeArtwork != 0, errorCode: errorCode)
    if errorCode?.pointee != 0 { return nil }
    let selected: [String: Any]?
    if spotify?["state"] as? String == "playing" { selected = spotify }
    else if music?["state"] as? String == "playing" { selected = music }
    else { selected = spotify ?? music }
    guard let selected,
          let data = try? JSONSerialization.data(withJSONObject: selected),
          let text = String(data: data, encoding: .utf8)
    else { return nil }
    return strdup(text)
}

@_cdecl("neosicht_music_transport")
public func neosichtMusicTransport(_ application: Int32, _ action: Int32) -> Int32 {
    let app = application == 0 ? "Spotify" : "Music"
    let command = action == 0 ? "previous track" : action == 1 ? "playpause" : "next track"
    var errorCode: Int32 = 0
    let result = runScript("tell application \"\(app)\" to \(command)", errorCode: &errorCode)
    return result == nil ? (errorCode == 1 ? 1 : 2) : 0
}

@_cdecl("neosicht_music_free_string")
public func neosichtMusicFreeString(_ value: UnsafeMutablePointer<CChar>?) {
    free(value)
}
