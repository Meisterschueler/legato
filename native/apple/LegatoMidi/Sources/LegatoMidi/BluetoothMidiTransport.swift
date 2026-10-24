import CoreBluetooth
import Foundation
import LegatoEngineC

/// Bluetooth MIDI via CoreBluetooth: scan for the BLE MIDI service and connect.
/// After the connection CoreMIDI exposes the device; device_added comes from CoreMidiTransport.
public final class BluetoothMidiTransport: NSObject, CBCentralManagerDelegate {
    static let midiService = CBUUID(string: "03B80E5A-EDE8-4B33-A751-6CE34EC4C700")
    private static let storedIdentifierKey = "legato.bluetooth.identifier"

    private let queue = DispatchQueue(label: "legato.bluetooth")
    private var central: CBCentralManager?
    private var scanning = false
    private var peripherals: [UUID: CBPeripheral] = [:]
    private var pending: UUID?

    /// Identifier of the last selected device; survives app restarts.
    public var storedIdentifier: String? {
        UserDefaults.standard.string(forKey: Self.storedIdentifierKey)
    }

    func start() {
        queue.async {
            self.central = CBCentralManager(delegate: self, queue: self.queue)
        }
    }

    func setScanning(_ enabled: Bool) {
        queue.async {
            self.scanning = enabled
            self.applyScan()
        }
    }

    /// Pending connection request; CoreBluetooth never times it out.
    func connect(identifier: String) {
        queue.async {
            guard let uuid = UUID(uuidString: identifier) else { return }
            UserDefaults.standard.set(identifier, forKey: Self.storedIdentifierKey)
            self.pending = uuid
            self.connectPending()
        }
    }

    private func applyScan() {
        guard let central, central.state == .poweredOn else { return }
        if scanning {
            central.scanForPeripherals(withServices: [Self.midiService])
        } else {
            central.stopScan()
        }
    }

    private func connectPending() {
        guard let central, central.state == .poweredOn, let uuid = pending else { return }
        guard let peripheral = peripherals[uuid] ?? central.retrievePeripherals(withIdentifiers: [uuid]).first else {
            return
        }
        peripherals[uuid] = peripheral
        central.connect(peripheral)
    }

    public func centralManagerDidUpdateState(_ central: CBCentralManager) {
        applyScan()
        connectPending()
    }

    public func centralManager(
        _ central: CBCentralManager,
        didDiscover peripheral: CBPeripheral,
        advertisementData: [String: Any],
        rssi RSSI: NSNumber
    ) {
        peripherals[peripheral.identifier] = peripheral
        let name = advertisementData[CBAdvertisementDataLocalNameKey] as? String ?? peripheral.name ?? "MIDI"
        peripheral.identifier.uuidString.withCString { identifier in
            name.withCString { legato_midi_bluetooth_found(identifier, $0) }
        }
    }

    public func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        if pending == peripheral.identifier {
            pending = nil
        }
    }

    public func centralManager(
        _ central: CBCentralManager,
        didFailToConnect peripheral: CBPeripheral,
        error: Error?
    ) {
        connectPending()
    }
}
