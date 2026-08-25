plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.{{superdev:project-compact}}.app"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.{{superdev:project-compact}}.app"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
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

    testOptions {
        unitTests {
            // Robolectric parses the merged manifest and resources to run framework code on the JVM.
            isIncludeAndroidResources = true
        }
    }
}

// Robolectric 4.14.1 pins Conscrypt 2.5.2, whose JNI bundle contains only linux-x86_64 and
// osx-x86_64 — so Robolectric cannot even start its test environment on ARM64 Linux (this
// devcontainer, on an Apple Silicon host): it dies in setUpApplicationState with
// "no conscrypt_openjdk_jni-linux-aarch_64 in java.library.path". 2.6.1 adds that native.
configurations.matching { it.name.contains("test", ignoreCase = true) }.configureEach {
    resolutionStrategy.force("org.conscrypt:conscrypt-openjdk-uber:2.6.1")
}

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.10.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")

    testImplementation("junit:junit:4.13.2")
    testImplementation("org.jetbrains.kotlin:kotlin-test")

    // Compose UI tests run under Robolectric as part of the normal `./gradlew test`, so no
    // device or emulator is needed and CI covers them. Robolectric runs the real AOSP
    // framework classes on the JVM (downloading an android-all jar on first use, then cached)
    // with shadows for anything backed by native code.
    testImplementation(platform("androidx.compose:compose-bom:2024.10.00"))
    testImplementation("androidx.compose.ui:ui-test-junit4")
    testImplementation("org.robolectric:robolectric:4.14.1")

    // Supplies the empty activity createComposeRule() hosts its content in.
    debugImplementation("androidx.compose.ui:ui-test-manifest")

    debugImplementation(project(":native-debug-server"))
}
