@Suppress("NOTHING_TO_INLINE")
private inline fun version(pkg: String, version: String): ((String) -> String) =
    { mod: String -> listOf(pkg, mod, version).joinToString(":") }

object Lib {
    object Square {
        private val wire = version("com.squareup.wire", "4.0.1")
        private val okHttp = version("com.squareup.okhttp3", "4.9.3")

        val wireRuntime = wire("wire-runtime")
        val wireGrpcClient = wire("wire-grpc-client")

        val okHttpClient = okHttp("okhttp")
    }

    object Kotlinx {
        private val kotlinx = version("org.jetbrains.kotlinx", "1.6.0")

        val coroutinesCore = kotlinx("kotlinx-coroutines-core")
    }
}