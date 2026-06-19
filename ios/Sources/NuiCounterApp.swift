import SwiftUI

@main
struct NuiCounterApp: App {
    @StateObject private var bridge = Bridge()

    var body: some Scene {
        WindowGroup {
            // CounterView is generated from examples/counter.nui by `nuic`.
            CounterView(bridge: bridge)
                .overlay(alignment: .top) {
                    ConnectionDot(connected: bridge.isConnected)
                }
                .onAppear {
                    bridge.connect()
                    // Dev affordance: `-nui-autodrive` proves the event path
                    // in-process when we can't tap the simulator programmatically.
                    if CommandLine.arguments.contains("-nui-autodrive") {
                        for i in 1...5 {
                            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5 * Double(i)) {
                                bridge.send(event: "increment")
                            }
                        }
                    }
                }
        }
    }
}

/// Dev-only chrome (not part of the generated UI): shows whether the logic is
/// connected, so a flat `0` doesn't look like a bug.
private struct ConnectionDot: View {
    let connected: Bool
    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(connected ? Color.green : Color.orange)
                .frame(width: 8, height: 8)
            Text(connected ? "connected to logic" : "connecting to logic…")
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(.top, 8)
    }
}
