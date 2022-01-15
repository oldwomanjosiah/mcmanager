package com.oldwomanjosiah.mcmanager.data

import com.squareup.wire.GrpcClient
import okhttp3.OkHttpClient
import okhttp3.Protocol

const val SERVER_URL = "http://127.0.0.1:50051"

fun getClient() = GrpcClient.Builder()
    .client(
        OkHttpClient
            .Builder()
            .protocols(listOf(Protocol.H2_PRIOR_KNOWLEDGE))
            .addInterceptor {
                val request = it.request()
                println("Making Request to ${request.url}")
                val resp = it.proceed(request)
                println("Got Response from ${request.url}")
                resp
            }
            .build()
    )
    .baseUrl(SERVER_URL)
    .build()