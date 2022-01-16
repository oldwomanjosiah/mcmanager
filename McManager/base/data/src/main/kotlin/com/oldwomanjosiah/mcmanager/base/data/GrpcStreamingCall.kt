package com.oldwomanjosiah.mcmanager.base.data

import com.squareup.wire.GrpcStreamingCall
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.consumeAsFlow
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.launch
import kotlinx.coroutines.plus

/**
 * Execute this as a single request, streaming response call
 *
 * May not execute the [GrpcStreamingCall] again after this call is made
 */
suspend fun <S: Any, R: Any> GrpcStreamingCall<S, R>.startReceiving(
    coroutineScope: CoroutineScope,
    request: S
): Flow<R> {
    val (req, resp) = executeIn(coroutineScope + Dispatchers.IO)

    req.send(request)
    req.close()

    return resp.consumeAsFlow()
}

/**
 * Execute this as a streaming request, single response call
 *
 * Suspends until request finishes
 *
 * May not execute the [GrpcStreamingCall] again after this call is made
 */
suspend fun <S: Any, R: Any> GrpcStreamingCall<S, R>.startSending(
    coroutineScope: CoroutineScope,
    request: Flow<S>,
): R {
    val (req, resp) = executeIn(coroutineScope + Dispatchers.IO)

    request
        .onCompletion { req.close() }
        .collect { req.send(it) }

    val r = resp.receive()
    resp

    return r
}

/**
 * Execute this as a bidirectional streaming call
 *
 * stream sends and receives may be interleaved
 */
suspend fun <S: Any, R: Any> GrpcStreamingCall<S, R>.startBidirectional(
    coroutineScope: CoroutineScope,
    request: Flow<S>
): Flow<R> {
    val (req, resp) = executeIn(coroutineScope + Dispatchers.IO)

    coroutineScope.launch {
        request.onCompletion { req.close() }
            .collect { req.send(it) }
    }

    return resp.consumeAsFlow()
}