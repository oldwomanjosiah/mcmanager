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
import com.oldwomanjosiah.mcmanager.base.models.Presenter
import com.oldwomanjosiah.mcmanager.base.models.launchState
import com.oldwomanjosiah.mcmanager.data.getClient
import com.oldwomanjosiah.mcmanager.helloworld.HelloRequest
import com.oldwomanjosiah.mcmanager.helloworld.HelloWorldServiceClient
import dispatch.core.DefaultCoroutineScope
import dispatch.core.defaultDispatcher
import dispatch.core.withDefault
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*

data class AppViewState(
    val responses: List<String>
)

class AppViewModel(
    coroutinesScope: CoroutineScope
) : Presenter<AppViewState>(coroutinesScope) {

    private val newGreetings = MutableSharedFlow<String>(extraBufferCapacity = 10)

    override val state = launchState {
        var responses by remember { mutableStateOf(listOf<String>()) }

        LaunchedEffect(Unit) {
            newGreetings.collect { responses += it }
        }

        AppViewState(responses = responses)
    }

    val client = getClient()
    val helloWorld: HelloWorldServiceClient = client.create()

    fun getGreeting(name: String) {
        coroutineScope.launch {
            newGreetings.emit(
                helloWorld
                    .HelloWorld()
                    .execute(HelloRequest(name = name))
                    .greeting
            )
        }
    }
}

@Composable
@Preview
fun App() {
    val coroutineScope = rememberCoroutineScope()
    val viewModel = remember { AppViewModel(coroutineScope) }
    var currentName by remember { mutableStateOf("") }

    val state = viewModel.collectState()

    MaterialTheme {
        Column(Modifier.padding(24.dp)) {
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

            state.responses.forEach { greeting ->
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
        App()
    }
}