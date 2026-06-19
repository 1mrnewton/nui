import Foundation

typealias PyObjectPtr = UnsafeMutablePointer<PyObject>

/// The nui runtime bridge — Phase 3, fully in-process.
///
/// No socket, no desktop server: the Python logic core (`counter.py`) is
/// embedded in the app and called directly through the CPython C API. The public
/// surface (`onState` / `send` / `connect` / `isConnected`) is IDENTICAL to the
/// socket bridge of Phases 0-2, so the generated UI compiles unchanged. Only the
/// transport swapped — "stable contract, swappable transport".
///
/// All Python interaction happens on a dedicated serial queue, GIL-guarded.
final class Bridge: ObservableObject {
    @Published private(set) var isConnected: Bool = false

    private let queue = DispatchQueue(label: "dev.nui.python")
    private var module: PyObjectPtr?
    private var stateHandlers: [(Data) -> Void] = []
    private var latestState: Data?

    /// Register a typed state subscriber. Generated models call this.
    func onState<T: Decodable>(_ handler: @escaping (T) -> Void) {
        let wrapped: (Data) -> Void = { data in
            guard let decoded = try? JSONDecoder().decode(T.self, from: data) else { return }
            DispatchQueue.main.async { handler(decoded) }
        }
        queue.async {
            self.stateHandlers.append(wrapped)
            if let latest = self.latestState { wrapped(latest) }
        }
    }

    /// Emit an event into the embedded logic and deliver the new state.
    func send(event name: String, payload: [String: Any] = [:]) {
        queue.async {
            let payloadJSON = Self.jsonString(payload)
            if let json = self.callString("dispatch_json", [name, payloadJSON]) {
                self.deliver(json)
            }
        }
    }

    /// Boot the interpreter, import the logic, deliver the initial state.
    func connect() {
        queue.async {
            PythonRuntime.initialize()
            self.module = PythonRuntime.withGIL { () -> PyObjectPtr? in
                guard let imported = PyImport_ImportModule("counter") else {
                    PyErr_Print()
                    return nil
                }
                return imported // retained for the app's lifetime
            }
            let ok = self.module != nil
            DispatchQueue.main.async { self.isConnected = ok }
            if let json = self.callString("initial_json", []) {
                self.deliver(json)
            }
        }
    }

    // MARK: - Python calls (on `queue`, GIL-guarded)

    private func callString(_ function: String, _ args: [String]) -> String? {
        guard let module else { return nil }
        return PythonRuntime.withGIL { () -> String? in
            guard let fn = PyObject_GetAttrString(module, function) else { return nil }
            defer { Py_DecRef(fn) }
            guard PyCallable_Check(fn) != 0 else { return nil }

            let result: PyObjectPtr?
            if args.isEmpty {
                result = PyObject_CallObject(fn, nil)
            } else {
                let tuple = PyTuple_New(args.count)
                for (index, arg) in args.enumerated() {
                    // PyTuple_SetItem steals the reference to the new string.
                    PyTuple_SetItem(tuple, index, PyUnicode_FromString(arg))
                }
                result = PyObject_CallObject(fn, tuple)
                Py_DecRef(tuple)
            }

            guard let result else { PyErr_Print(); return nil }
            defer { Py_DecRef(result) }
            guard let cString = PyUnicode_AsUTF8(result) else { return nil }
            return String(cString: cString)
        }
    }

    private func deliver(_ json: String) {
        guard let data = json.data(using: .utf8) else { return }
        latestState = data
        for handler in stateHandlers { handler(data) }
    }

    private static func jsonString(_ payload: [String: Any]) -> String {
        guard !payload.isEmpty,
              let data = try? JSONSerialization.data(withJSONObject: payload),
              let string = String(data: data, encoding: .utf8)
        else { return "{}" }
        return string
    }
}
