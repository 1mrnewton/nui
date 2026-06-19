package dev.nui

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import dev.nui.generated.CounterView
import dev.nui.runtime.Bridge

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val bridge = Bridge()
        bridge.connect()
        // Dev: adb shell am start -n dev.nui.counter/dev.nui.MainActivity --ez nui-autodrive true
        if (intent.getBooleanExtra("nui-autodrive", false)) {
            val main = Handler(Looper.getMainLooper())
            repeat(5) { i ->
                main.postDelayed({ bridge.send("increment") }, 500L * (i + 1))
            }
        }
        setContent {
            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    // CounterView is generated from examples/counter.nui by `nuic`.
                    CounterView(bridge = bridge)
                }
            }
        }
    }
}
