import Foundation
import LegatoEngineC

#if os(iOS)
import AVFoundation
import AudioToolbox
#endif

/// Native MIDI layer for iOS and macOS (CoreMIDI, CoreBluetooth); see contracts/native-midi.md.
public final class LegatoMidi {
    public static let shared = LegatoMidi()

    let coreMidi = CoreMidiTransport()
    let bluetooth = BluetoothMidiTransport()

    private init() {}

    /// Called on the CoreMIDI thread for every packet: device, driver time and arrival time in µs.
    public var onTrace: ((UInt32, UInt64, UInt64) -> Void)? {
        get { coreMidi.onTrace }
        set { coreMidi.onTrace = newValue }
    }

    /// Registers the platform callbacks with the engine and starts both transports.
    public func start() {
        var platform = LegatoMidiPlatform(
            connect: { LegatoMidi.shared.coreMidi.connect(device: $0) },
            disconnect: { LegatoMidi.shared.coreMidi.disconnect(device: $0) },
            send: { device, data, length in
                guard let data else { return }
                let bytes = Array(UnsafeBufferPointer(start: data, count: length))
                LegatoMidi.shared.coreMidi.send(device: device, bytes: bytes)
            },
            bluetooth_scan: { LegatoMidi.shared.bluetooth.setScanning($0) },
            bluetooth_connect: { identifier in
                guard let identifier else { return }
                LegatoMidi.shared.bluetooth.connect(identifier: String(cString: identifier))
            },
            prepare_audio_session: { LegatoMidi.prepareAudioSession() },
            vibrate: { LegatoMidi.vibrate() }
        )
        legato_midi_register(&platform)
        coreMidi.start()
        bluetooth.start()
    }

    public var storedBluetoothIdentifier: String? { bluetooth.storedIdentifier }

    // Debug interface of the engine, used by the spike apps.

    public func scanBluetooth(_ enabled: Bool) {
        legato_debug_scan_bluetooth(enabled)
    }

    public func selectBluetooth(identifier: String) {
        identifier.withCString { legato_debug_select_bluetooth($0) }
    }

    /// Engine log entries (one JSON object per line) since the last call.
    public func drainEngineLog() -> [String] {
        var lines: [String] = []
        var buffer = [UInt8](repeating: 0, count: 64 * 1024)
        while true {
            let count = buffer.withUnsafeMutableBufferPointer { legato_debug_drain($0.baseAddress, $0.count) }
            if count == 0 { break }
            lines += String(decoding: buffer[..<count], as: UTF8.self).split(separator: "\n").map(String.init)
        }
        return lines
    }

    static func prepareAudioSession() {
        #if os(iOS)
        try? AVAudioSession.sharedInstance().setCategory(.playback, options: [.mixWithOthers])
        try? AVAudioSession.sharedInstance().setActive(true)
        #endif
    }

    static func vibrate() {
        #if os(iOS)
        AudioServicesPlaySystemSound(kSystemSoundID_Vibrate)
        #endif
    }
}
