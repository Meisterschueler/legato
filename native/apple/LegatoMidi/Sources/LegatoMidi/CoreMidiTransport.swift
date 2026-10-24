import CoreMIDI
import Foundation
import LegatoEngineC

/// MIDI sources and destinations via CoreMIDI (USB, and Bluetooth devices once connected).
/// Device ids are the CoreMIDI unique ids of the sources.
public final class CoreMidiTransport {
    private var client = MIDIClientRef()
    private var inputPort = MIDIPortRef()
    private var outputPort = MIDIPortRef()
    private var online: [UInt32: MIDIEndpointRef] = [:]
    private let queue = DispatchQueue(label: "legato.coremidi")

    /// Called on the CoreMIDI thread for every packet: device, driver time and arrival time in µs.
    var onTrace: ((UInt32, UInt64, UInt64) -> Void)?

    func start() {
        MIDIClientCreateWithBlock("Legato" as CFString, &client) { [weak self] _ in
            self?.queue.async { self?.rescan() }
        }
        MIDIInputPortCreateWithBlock(client, "Legato In" as CFString, &inputPort) { [weak self] list, refCon in
            self?.receive(list, device: UInt32(truncatingIfNeeded: UInt(bitPattern: refCon)))
        }
        MIDIOutputPortCreate(client, "Legato Out" as CFString, &outputPort)
        queue.async { self.rescan() }
    }

    func connect(device: UInt32) {
        queue.async {
            guard let source = self.online[device] else { return }
            MIDIPortConnectSource(self.inputPort, source, UnsafeMutableRawPointer(bitPattern: UInt(device)))
        }
    }

    func disconnect(device: UInt32) {
        queue.async {
            guard let source = self.online[device] else { return }
            MIDIPortDisconnectSource(self.inputPort, source)
        }
    }

    func send(device: UInt32, bytes: [UInt8]) {
        queue.async {
            guard let source = self.online[device], let destination = Self.destination(of: source) else { return }
            var buffer = [UInt8](repeating: 0, count: 1024)
            buffer.withUnsafeMutableBytes { raw in
                let list = raw.baseAddress!.assumingMemoryBound(to: MIDIPacketList.self)
                let packet = MIDIPacketListInit(list)
                MIDIPacketListAdd(list, raw.count, packet, 0, bytes.count, bytes)
                MIDISend(self.outputPort, destination, list)
            }
        }
    }

    /// Compares the online sources with the known ones and reports the difference to the engine.
    private func rescan() {
        var current: [UInt32: MIDIEndpointRef] = [:]
        for index in 0..<MIDIGetNumberOfSources() {
            let source = MIDIGetSource(index)
            if Self.integer(source, kMIDIPropertyOffline) == 0 {
                current[UInt32(bitPattern: Self.integer(source, kMIDIPropertyUniqueID))] = source
            }
        }
        for device in online.keys where current[device] == nil {
            legato_midi_device_removed(device)
        }
        for (device, source) in current where online[device] == nil {
            let name = Self.string(source, kMIDIPropertyDisplayName) ?? "MIDI"
            let transport = Self.isBluetooth(source) ? LEGATO_TRANSPORT_BLUETOOTH : LEGATO_TRANSPORT_USB
            name.withCString { legato_midi_device_added(device, $0, UInt8(transport)) }
        }
        online = current
    }

    private func receive(_ list: UnsafePointer<MIDIPacketList>, device: UInt32) {
        let arrival = HostTime.microseconds(mach_absolute_time())
        for packet in list.unsafeSequence() {
            let stamp = packet.pointee.timeStamp
            let time = stamp == 0 ? arrival : HostTime.microseconds(stamp)
            let length = Int(packet.pointee.length)
            // MIDIPacket is variable-sized; read the bytes in place instead of copying the struct.
            let data = UnsafeRawPointer(packet)
                .advanced(by: MemoryLayout<MIDIPacket>.offset(of: \MIDIPacket.data)!)
                .assumingMemoryBound(to: UInt8.self)
            legato_midi_message(device, time, data, length)
            onTrace?(device, time, arrival)
        }
    }

    private static func destination(of source: MIDIEndpointRef) -> MIDIEndpointRef? {
        var entity = MIDIEntityRef()
        guard MIDIEndpointGetEntity(source, &entity) == noErr, MIDIEntityGetNumberOfDestinations(entity) > 0 else {
            return nil
        }
        return MIDIEntityGetDestination(entity, 0)
    }

    private static func isBluetooth(_ source: MIDIEndpointRef) -> Bool {
        var entity = MIDIEntityRef()
        var device = MIDIDeviceRef()
        guard MIDIEndpointGetEntity(source, &entity) == noErr, MIDIEntityGetDevice(entity, &device) == noErr else {
            return false
        }
        return string(device, kMIDIPropertyDriverOwner)?.contains("Bluetooth") ?? false
    }

    private static func integer(_ object: MIDIObjectRef, _ property: CFString) -> Int32 {
        var value: Int32 = 0
        MIDIObjectGetIntegerProperty(object, property, &value)
        return value
    }

    private static func string(_ object: MIDIObjectRef, _ property: CFString) -> String? {
        var value: Unmanaged<CFString>?
        guard MIDIObjectGetStringProperty(object, property, &value) == noErr else { return nil }
        return value?.takeRetainedValue() as String?
    }
}

/// Conversion of host time (mach_absolute_time, MIDITimeStamp) to microseconds.
public enum HostTime {
    private static let timebase: mach_timebase_info_data_t = {
        var info = mach_timebase_info_data_t()
        mach_timebase_info(&info)
        return info
    }()

    public static func microseconds(_ hostTime: UInt64) -> UInt64 {
        UInt64(Double(hostTime) * Double(timebase.numer) / Double(timebase.denom) / 1000)
    }

    public static var nowMicroseconds: UInt64 { microseconds(mach_absolute_time()) }
}
