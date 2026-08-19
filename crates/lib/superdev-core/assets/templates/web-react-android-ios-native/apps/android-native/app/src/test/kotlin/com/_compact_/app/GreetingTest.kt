package com.{{superdev:project-compact}}.app

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/**
 * A Compose UI test on the JVM: Robolectric hosts the composition, so this
 * runs in plain `./gradlew test` with no device or emulator.
 */
@RunWith(RobolectricTestRunner::class)
class GreetingTest {

    @get:Rule
    val compose = createComposeRule()

    @Test
    fun greetingIsDisplayed() {
        compose.setContent { Greeting() }
        compose.onNodeWithText("Hello, world").assertIsDisplayed()
    }
}
