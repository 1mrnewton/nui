import Foundation

/// Owns the embedded CPython interpreter lifecycle and the GIL.
///
/// The interpreter is initialized once. After init we release the GIL (via
/// `PyEval_SaveThread`) so that any worker thread can safely acquire it with
/// `PyGILState_Ensure` / `PyGILState_Release` around each Python interaction.
enum PythonRuntime {
    private static var initialized = false

    static func initialize() {
        guard !initialized else { return }
        guard let resourcePath = Bundle.main.resourcePath else {
            print("[python] no resourcePath")
            return
        }

        let home = "\(resourcePath)/python"
        let lib = "\(home)/lib/python3.13"
        let dynload = "\(lib)/lib-dynload"
        let appDir = "\(resourcePath)/app"

        setenv("PYTHONHOME", home, 1)
        setenv("PYTHONPATH", "\(lib):\(dynload):\(appDir)", 1)
        setenv("PYTHONDONTWRITEBYTECODE", "1", 1)
        setenv("PYTHONUNBUFFERED", "1", 1)

        Py_Initialize()
        // Drop the GIL acquired by Py_Initialize; workers re-acquire per call.
        _ = PyEval_SaveThread()
        initialized = true
        print("[python] interpreter initialized")
    }

    /// Run a block with the GIL held. Safe to call from any thread.
    static func withGIL<T>(_ body: () -> T) -> T {
        let gil = PyGILState_Ensure()
        defer { PyGILState_Release(gil) }
        return body()
    }
}
