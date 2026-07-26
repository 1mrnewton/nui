import SwiftUI

@main
struct CounterApp: App {
    var body: some Scene {
        WindowGroup {
            CounterView(store: CounterStore(logic: RustCounterLogic()))
        }
    }
}
