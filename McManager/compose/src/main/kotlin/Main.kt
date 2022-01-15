// Copyright 2000-2021 JetBrains s.r.o. and contributors. Use of this source code is governed by the Apache 2.0 license that can be found in the LICENSE file.
import androidx.compose.animation.core.snap
import androidx.compose.desktop.ui.tooling.preview.Preview
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.*
import androidx.compose.runtime.*
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import com.oldwomanjosiah.mcmanager.base.models.Presenter
import com.oldwomanjosiah.mcmanager.base.models.launchState
import com.oldwomanjosiah.mcmanager.data.getClient
import com.oldwomanjosiah.mcmanager.event.Event
import com.oldwomanjosiah.mcmanager.event.EventSubscription
import com.oldwomanjosiah.mcmanager.event.EventsClient
import com.oldwomanjosiah.mcmanager.helloworld.HelloRequest
import com.oldwomanjosiah.mcmanager.helloworld.HelloWorldServiceClient
import dispatch.core.withIO
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*

data class AppViewState(
    val responses: List<String>,
    val events: List<Event>
)

class AppViewModel(
    coroutinesScope: CoroutineScope
) : Presenter<AppViewState>(coroutinesScope) {

    private val newGreetings = MutableSharedFlow<String>(extraBufferCapacity = 10)

    override val state = launchState {
        var responses by remember { mutableStateOf(listOf<String>()) }
        var events by remember { mutableStateOf(listOf<Event>()) }

        LaunchedEffect(Unit) {
            println("Pre exe")
            val (request, resp) = eventClient.Subscribe().executeIn(this + Dispatchers.IO)

            println("Starting Send")

            request.send(EventSubscription(1))
            request.close()

            println("Starting Recv")

            launch {
                listOf(
                    resp.consumeAsFlow()
                        .catch { println("Failed to make request: $it") }
                        .onEach { events += it },
                    newGreetings.onEach { responses += it }
                ).merge().collect()
            }

            //println(eventClient.Snapshot().execute(EventSubscription(1)).toString())
        }

        AppViewState(responses = responses, events = events)
    }

    val client = getClient()
    val helloWorld: HelloWorldServiceClient = client.create()
    val eventClient: EventsClient = client.create()

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
                TextField(
                    value = currentName, onValueChange = { currentName = it }, modifier = Modifier.padding(
                        PaddingValues(end = 12.dp)
                    )
                )
                Button(onClick = {
                    viewModel.getGreeting(currentName)
                }) {
                    Text("Submit")
                }
            }

            Spacer(modifier = Modifier.padding(PaddingValues(bottom = 24.dp)))

            Row(Modifier.fillMaxWidth()) {
                Column(Modifier.weight(1f).verticalScroll(rememberScrollState())) {
                    state.responses.forEach { greeting ->
                        Text(greeting, modifier = Modifier.padding(PaddingValues(bottom = 12.dp)))
                    }
                }

                LazyColumn(
                    Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(state.events) { event ->
                        Card(Modifier.fillMaxWidth(), elevation = 8.dp) {
                            event.system_snapshot?.let { snapshot ->
                                Column {
                                    Text("Cpu: ${snapshot.cpu_pressure}")
                                    Text("Memory: ${snapshot.mem_pressure}")
                                }
                            }
                        }
                    }
                }
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