package com.{{superdev:project-compact}}.app

import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // No-op in release builds: the debug source set carries the real
        // implementations, the release source set carries stubs.
        DebugServerInit.start(this)
        DebugBridge.attach()
        enableEdgeToEdge()
        setContent {
            MaterialTheme {
                Greeting()
            }
        }
    }

    override fun onDestroy() {
        DebugServerInit.stop()
        super.onDestroy()
    }
}

@Composable
fun Greeting(modifier: Modifier = Modifier) {
    Surface(modifier = modifier.fillMaxSize()) {
        Text(
            text = "Hello, world",
            style = MaterialTheme.typography.headlineMedium,
            modifier = Modifier
                .fillMaxSize()
                .wrapContentSize(Alignment.Center)
        )
    }
}
