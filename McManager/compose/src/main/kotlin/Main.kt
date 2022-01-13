// Copyright 2000-2021 JetBrains s.r.o. and contributors. Use of this source code is governed by the Apache 2.0 license that can be found in the LICENSE file.
import androidx.compose.material.MaterialTheme
import androidx.compose.desktop.ui.tooling.preview.Preview
import androidx.compose.foundation.layout.*
import androidx.compose.material.Button
import androidx.compose.material.Text
import androidx.compose.material.TextField
import androidx.compose.runtime.*
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.*
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.MenuBar
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import com.oldwomanjosiah.mcmanager.data.getClient
import com.oldwomanjosiah.mcmanager.helloworld.HelloRequest
import com.oldwomanjosiah.mcmanager.helloworld.HelloWorldServiceClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlin.coroutines.CoroutineContext

class AppViewModel(
    val coroutinesScope: CoroutineScope
) {
    val client = getClient()
    val helloWorld = client.create<HelloWorldServiceClient>()

    private var responses = listOf<String>()

    private val _responseFlow = MutableStateFlow(responses)
    val reponseFlow: StateFlow<List<String>> = _responseFlow

    fun getGreeting(name: String) = coroutinesScope.launch {
        responses += helloWorld.HelloWorld().execute(HelloRequest(name = name)).greeting
        _responseFlow.emit(responses)
    }
}

@Composable
@Preview
fun App() {
    var text by remember { mutableStateOf("Hello, World!") }
    val coroutineScope = rememberCoroutineScope()
    val viewModel = remember { AppViewModel(coroutineScope) }
    var currentName by remember { mutableStateOf("") }
    val greetings by viewModel.reponseFlow.collectAsState(listOf())

    MaterialTheme {
        Column {
            Row {
                TextField(value = currentName, onValueChange = { currentName = it }, modifier = Modifier.padding(
                    PaddingValues(end = 12.dp)
                ))
                Button(onClick = {
                    viewModel.getGreeting(currentName)
                }) {
                    Text("Submit")
                }
            }
            Spacer(modifier = Modifier.padding(PaddingValues(bottom = 24.dp)))

            greetings.forEach { greeting ->
                Text(greeting, modifier = Modifier.padding(PaddingValues(bottom = 12.dp)))
            }
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
fun main() = application {

    Window(
        onCloseRequest = ::exitApplication,
    ) {
        MenuBar {
            Menu("McManager", mnemonic = 'M') {
                Item("Quit", shortcut = KeyShortcut(Key.Q, ctrl = true), onClick = ::exitApplication)
            }
        }

        App()
    }
}