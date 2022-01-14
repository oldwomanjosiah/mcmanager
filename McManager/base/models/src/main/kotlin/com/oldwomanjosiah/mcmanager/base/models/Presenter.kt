package com.oldwomanjosiah.mcmanager.base.models

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import com.oldwomanjosiah.mcmanager.base.models.molecule.launchMolecule
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.*

abstract class Presenter<S : Any?>(
    val coroutineScope: CoroutineScope
) {

    abstract val state: StateFlow<S>

    /**
     * Collect the current state flow in this composition
     */
    @Composable
    fun collectState(): S = state.collectAsState(state.value).value
}

inline fun <reified S> Presenter<S>.launchState(noinline body: @Composable () -> S): StateFlow<S> = coroutineScope.launchMolecule(body)
