import java.io.File

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.kotlin.plugin.serialization")
    id("com.chaquo.python")
}

android {
    namespace = "dev.nui"
    compileSdk = 36

    defaultConfig {
        applicationId = "dev.nui.counter"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = "0.0.1"
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    buildTypes {
        getByName("debug") {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
    buildFeatures {
        compose = true
    }
}

chaquopy {
    defaultConfig {
        version = "3.13"
        val python313 = sequenceOf(
            "/opt/homebrew/opt/python@3.13/bin/python3.13",
            "/usr/local/opt/python@3.13/bin/python3.13",
        ).firstOrNull { File(it).exists() } ?: "python3.13"
        buildPython(python313)
        pyc {
            // Keep .py in the APK during dev so stack traces show source.
            src = false
        }
    }
}

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.12.01"))
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
}

// Keep the embedded logic in sync with logic/counter.py (same as iOS pre-build step).
tasks.register("syncLogic") {
    doLast {
        val src = file("../../logic/counter.py")
        val dest = file("src/main/python/counter.py")
        dest.parentFile.mkdirs()
        src.copyTo(dest, overwrite = true)
    }
}
tasks.named("preBuild").configure { dependsOn("syncLogic") }
