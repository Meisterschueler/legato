// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "LegatoMidi",
    platforms: [.iOS(.v16), .macOS(.v12)],
    products: [
        .library(name: "LegatoMidi", targets: ["LegatoMidi"]),
    ],
    targets: [
        // C declarations of the engine interface (contracts/native-midi.md).
        .target(name: "LegatoEngineC"),
        .target(name: "LegatoMidi", dependencies: ["LegatoEngineC"]),
    ]
)
