@Suppress("NOTHING_TO_INLINE")
private inline fun version(pkg: String, version: String, modulePrefix: String? = null): ((String) -> String) =
    { mod: String ->
        val mod = modulePrefix?.let { "$it$mod" } ?: mod
        listOf(pkg, mod, version).joinToString(":")
    }

object Lib {
    object Square {
        private val wire = version("com.squareup.wire", "4.0.1", "wire-")
        private val okHttp = version("com.squareup.okhttp3", "4.9.3")

        val wireRuntime = wire("runtime")
        val wireGrpcClient = wire("grpc-client")

        val okHttpClient = okHttp("okhttp")
    }

    object Cash {
        private val turbine = version("app.cash.turbine", "0.6.0")

        val turbineTest = turbine("turbine")
    }

    object Kotlinx {
        private val kotlinx = version("org.jetbrains.kotlinx", "1.6.0", "kotlinx-")

        val coroutinesCore = kotlinx("coroutines-core")
    }

    object RBusarow {
        private val dispatch = version("com.rickbusarow.dispatch", "1.0.0-beta10", "dispatch-")

        val dispatchCore = dispatch("core")
        val dispatchTest = dispatch("test5")
    }
}