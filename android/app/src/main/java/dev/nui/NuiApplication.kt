package dev.nui

import android.app.Application
import com.chaquo.python.Python
import com.chaquo.python.android.AndroidPlatform

/** Boots the embedded CPython interpreter once per process (Chaquopy). */
class NuiApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        if (!Python.isStarted()) {
            Python.start(AndroidPlatform(this))
        }
    }
}
