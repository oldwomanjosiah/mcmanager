@Suppress("NOTHING_TO_INLINE")
private inline fun version(pkg: String, version: String): ((String) -> String) =
    { mod: String -> listOf(pkg, mod, version).joinToString(":") }

object Lib {
    object Square {
        private val wire = version("com.squareup.wire", "4.0.1")

        val wireRuntime = wire("wire-runtime")
        val wireGrpcClient = wire("wire-grpc-client")
    }
}