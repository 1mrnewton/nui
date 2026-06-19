// GENERATED from counter.nui by nuic — do not edit.
import SwiftUI

struct CounterState: Decodable {
    var count: Int = 0
    var step: Int = 1
    var show_label: Bool = true
    var title: String = "Counter"
}

@MainActor
final class CounterModel: ObservableObject {
    @Published private(set) var state = CounterState()
    private let bridge: Bridge

    init(bridge: Bridge) {
        self.bridge = bridge
        bridge.onState { [weak self] (new: CounterState) in self?.state = new }
    }

    func increment() { bridge.send(event: "increment") }

    func decrement() { bridge.send(event: "decrement") }

    func reset() { bridge.send(event: "reset") }

    func set_step(_ value: Int) { bridge.send(event: "set_step", payload: ["value": value]) }

    func set_show_label(_ value: Bool) { bridge.send(event: "set_show_label", payload: ["value": value]) }

    func set_title(_ value: String) { bridge.send(event: "set_title", payload: ["value": value]) }

}

struct CounterView: View {
    @StateObject private var model: CounterModel
    init(bridge: Bridge) { _model = StateObject(wrappedValue: CounterModel(bridge: bridge)) }

    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                HStack(spacing: 8) {
                    Image(systemName: "arrow.clockwise")
                    .foregroundStyle(.secondary)
                    .font(.system(size: 22))
                    Text("\(model.state.title)")
                    .font(.system(size: 28, weight: .bold))
                }
                .frame(maxWidth: .infinity)
                Group {
                    VStack(spacing: 16) {
                        Text("\(model.state.count)")
                        .font(.system(size: 72, weight: .bold))
                        if model.state.show_label {
                            Text("step by \(model.state.step)")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                        } else {
                            Text("subtitle hidden")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                        }
                        ProgressView(
                            value: Double(min(model.state.count, 20)),
                            total: 20
                        )
                        .frame(maxWidth: .infinity)
                        HStack(spacing: 20) {
                            Button(action: model.decrement) {
                                Image(systemName: "minus")
                            }
                            .tint(.red)
                            Button(action: model.increment) {
                                Image(systemName: "plus")
                            }
                            .tint(.green)
                        }
                        Button("Reset", action: model.reset)
                        .buttonStyle(.bordered)
                    }
                    .frame(maxWidth: .infinity)
                }
                .padding()
                .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 12))
                .frame(maxWidth: .infinity)
                Divider()
                VStack(spacing: 12) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Step")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                        Slider(
                            value: Binding(
                                get: { Double(model.state.step) },
                                set: { model.set_step(Int($0)) }
                            ),
                            in: 1...10,
                            step: 1
                        )
                    }
                    .frame(maxWidth: .infinity)
                    Toggle("Show subtitle", isOn: Binding(
                        get: { model.state.show_label },
                        set: { model.set_show_label($0) }
                    ))
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Title")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                        TextField("Title", text: Binding(
                            get: { model.state.title },
                            set: { model.set_title($0) }
                        ))
                        .textFieldStyle(.roundedBorder)
                    }
                    .frame(maxWidth: .infinity)
                }
            }
            .padding(24)
            .frame(maxWidth: .infinity)
        }
    }
}
