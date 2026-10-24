import SwiftUI

@main
struct LegatoS1App: App {
    @StateObject private var model = SpikeModel()

    var body: some Scene {
        WindowGroup {
            ContentView(model: model)
                .onAppear { model.start() }
        }
    }
}

struct ContentView: View {
    @ObservedObject var model: SpikeModel

    var body: some View {
        NavigationStack {
            List {
                Section("Bluetooth") {
                    Toggle("Suche", isOn: $model.scanning)
                    ForEach(model.found) { device in
                        Button(device.name) { model.select(device) }
                    }
                }
                Section("Verbunden") {
                    if model.connected.isEmpty {
                        Text("Kein Gerät").foregroundStyle(.secondary)
                    }
                    ForEach(model.connected.sorted(by: { $0.key < $1.key }), id: \.key) { entry in
                        Text(entry.value)
                    }
                }
                Section("Messwerte") {
                    Text(model.summary).font(.caption.monospaced())
                }
                Section("Protokoll") {
                    ForEach(Array(model.log.enumerated()), id: \.offset) { item in
                        Text(item.element).font(.caption.monospaced())
                    }
                }
            }
            .navigationTitle("Legato S1")
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Zurücksetzen") { model.reset() }
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    ShareLink(item: model.exportText) { Image(systemName: "square.and.arrow.up") }
                }
            }
        }
    }
}
