import Foundation
import LegatoMidi

struct FoundDevice: Identifiable, Hashable {
    let id: String
    let name: String
}

/// Collects engine log entries and timing traces and derives the S1 measurements.
final class SpikeModel: ObservableObject {
    @Published private(set) var log: [String] = []
    @Published private(set) var found: [FoundDevice] = []
    @Published private(set) var connected: [UInt32: String] = [:]
    @Published private(set) var messageCount = 0
    @Published private(set) var latenciesMs: [Double] = []
    @Published private(set) var removalGapsS: [Double] = []
    @Published private(set) var reconnectsS: [Double] = []
    @Published var scanning = false {
        didSet { LegatoMidi.shared.scanBluetooth(scanning) }
    }

    private var exportLines: [String] = []
    private var lastMessageUs: [UInt32: UInt64] = [:]
    private var removedAtUs: [UInt32: UInt64] = [:]
    private let traceLock = NSLock()
    private var pendingTraces: [(device: UInt32, timeUs: UInt64, arrivalUs: UInt64)] = []
    private var timer: Timer?

    func start() {
        guard timer == nil else { return }
        LegatoMidi.shared.onTrace = { [weak self] device, time, arrival in
            guard let self else { return }
            self.traceLock.lock()
            self.pendingTraces.append((device, time, arrival))
            self.traceLock.unlock()
        }
        LegatoMidi.shared.start()
        if let identifier = LegatoMidi.shared.storedBluetoothIdentifier {
            LegatoMidi.shared.selectBluetooth(identifier: identifier)
        }
        timer = Timer.scheduledTimer(withTimeInterval: 0.05, repeats: true) { [weak self] _ in self?.poll() }
    }

    func select(_ device: FoundDevice) {
        scanning = false
        LegatoMidi.shared.selectBluetooth(identifier: device.id)
        append("Auswahl \(device.name) (\(device.id))")
    }

    func reset() {
        messageCount = 0
        latenciesMs = []
        removalGapsS = []
        reconnectsS = []
        log = []
        exportLines = []
    }

    var summary: String {
        let latency = Stats(latenciesMs)
        return """
            Nachrichten: \(messageCount)
            Latenz (ms): Mittel \(f(latency.mean)), p95 \(f(latency.p95)), max \(f(latency.maximum))
            Jitter (ms, Standardabweichung der Latenz): \(f(latency.stdDev))
            Trennung erkannt nach letzter Nachricht (s): \(removalGapsS.map(f).joined(separator: ", "))
            Wiederverbunden nach Trennung (s): \(reconnectsS.map(f).joined(separator: ", "))
            """
    }

    var exportText: String {
        "Legato Spike S1 – \(Date())\n\n\(summary)\n\nProtokoll:\n" + exportLines.joined(separator: "\n")
    }

    private func poll() {
        traceLock.lock()
        let traces = pendingTraces
        pendingTraces.removeAll()
        traceLock.unlock()
        for trace in traces {
            latenciesMs.append(Double(Int64(trace.arrivalUs) - Int64(trace.timeUs)) / 1000)
            lastMessageUs[trace.device] = trace.arrivalUs
        }

        for line in LegatoMidi.shared.drainEngineLog() {
            guard let data = line.data(using: .utf8),
                let event = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                let kind = event["kind"] as? String
            else { continue }
            handle(kind: kind, event: event, line: line)
        }
    }

    private func handle(kind: String, event: [String: Any], line: String) {
        let now = HostTime.nowMicroseconds
        let device = (event["device"] as? NSNumber)?.uint32Value ?? 0
        switch kind {
        case "message":
            messageCount += 1
            exportLines.append("\(now) \(line)")
            if let bytes = event["data"] as? [Int], let status = bytes.first, status & 0xF0 == 0x90 {
                show("♪ \(bytes.map(String.init).joined(separator: " "))")
            }
            return
        case "device_added":
            let name = event["name"] as? String ?? "?"
            connected[device] = name
            if let removed = removedAtUs.removeValue(forKey: device) {
                let seconds = Double(now - removed) / 1_000_000
                reconnectsS.append(seconds)
                append("Wiederverbunden: \(name) nach \(f(seconds)) s")
            } else {
                append("Verbunden: \(name) (\(event["transport"] as? String ?? "?"))")
            }
        case "device_removed":
            let name = connected.removeValue(forKey: device) ?? "\(device)"
            removedAtUs[device] = now
            if let last = lastMessageUs[device] {
                let seconds = Double(now - last) / 1_000_000
                removalGapsS.append(seconds)
                append("Getrennt: \(name), erkannt \(f(seconds)) s nach letzter Nachricht")
            } else {
                append("Getrennt: \(name)")
            }
        case "bluetooth_found":
            let identifier = event["identifier"] as? String ?? ""
            if !found.contains(where: { $0.id == identifier }) {
                found.append(FoundDevice(id: identifier, name: event["name"] as? String ?? "MIDI"))
            }
        default:
            break
        }
        exportLines.append("\(now) \(line)")
    }

    private func append(_ text: String) {
        exportLines.append("\(HostTime.nowMicroseconds) # \(text)")
        show(text)
    }

    private func show(_ text: String) {
        log.insert(text, at: 0)
        if log.count > 300 { log.removeLast() }
    }
}

struct Stats {
    let mean: Double
    let p95: Double
    let maximum: Double
    let stdDev: Double

    init(_ values: [Double]) {
        let sorted = values.sorted()
        let count = Double(max(values.count, 1))
        let average = values.reduce(0, +) / count
        mean = average
        p95 = sorted.isEmpty ? 0 : sorted[min(sorted.count - 1, Int(Double(sorted.count) * 0.95))]
        maximum = sorted.last ?? 0
        stdDev = (values.map { ($0 - average) * ($0 - average) }.reduce(0, +) / count).squareRoot()
    }
}

func f(_ value: Double) -> String {
    String(format: "%.2f", value)
}
