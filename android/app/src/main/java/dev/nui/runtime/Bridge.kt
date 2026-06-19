package dev.nui.runtime

import android.os.Handler
import android.os.Looper
import android.util.Log
import com.chaquo.python.Python
import kotlinx.serialization.json.Json
import java.util.concurrent.Executors

/**
 * The nui runtime bridge — Phase 3, fully in-process (Android).
 *
 * Same public surface as iOS: `onState`, `send`, `connect`. Payload-bearing
 * events use the same JSON contract as the wire protocol.
 */
class Bridge {
    val json = Json { ignoreUnknownKeys = true }

    private val main = Handler(Looper.getMainLooper())
    private val handlers = mutableListOf<(String) -> Unit>()
    private var latest: String? = null
    private val queue = Executors.newSingleThreadExecutor()

    @Volatile
    var isConnected: Boolean = false
        private set

    fun addStateHandler(handler: (String) -> Unit) {
        handlers.add(handler)
        latest?.let { snapshot -> main.post { handler(snapshot) } }
    }

    inline fun <reified T> onState(crossinline handler: (T) -> Unit) {
        addStateHandler { raw -> handler(json.decodeFromString<T>(raw)) }
    }

    fun send(event: String, payload: Map<String, Any> = emptyMap()) {
        queue.execute {
            try {
                val counter = Python.getInstance().getModule("counter")
                val payloadJson = jsonPayload(payload)
                val stateJson = counter.callAttr("dispatch_json", event, payloadJson).toString()
                deliver(stateJson)
            } catch (e: Exception) {
                Log.e(TAG, "send($event) failed", e)
            }
        }
    }

    fun connect() {
        queue.execute {
            try {
                val counter = Python.getInstance().getModule("counter")
                isConnected = true
                val stateJson = counter.callAttr("initial_json").toString()
                deliver(stateJson)
            } catch (e: Exception) {
                Log.e(TAG, "connect failed", e)
                isConnected = false
            }
        }
    }

    private fun deliver(stateJson: String) {
        latest = stateJson
        val snapshot = handlers.toList()
        main.post { snapshot.forEach { it(stateJson) } }
    }

    private fun jsonPayload(payload: Map<String, Any>): String {
        if (payload.isEmpty()) return "{}"
        val parts = payload.map { (k, v) ->
            val encoded = when (v) {
                is String -> "\"${v.replace("\"", "\\\"")}\""
                is Boolean -> v.toString()
                is Int -> v.toString()
                is Long -> v.toString()
                is Double -> v.toString()
                else -> "\"$v\""
            }
            "\"$k\":$encoded"
        }
        return "{${parts.joinToString(",")}}"
    }

    companion object {
        private const val TAG = "nui.Bridge"
    }
}
