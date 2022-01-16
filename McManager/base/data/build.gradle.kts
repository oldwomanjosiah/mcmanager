plugins {
    kotlinJvm
}

dependencies {
    api(Lib.Square.wireRuntime)
    api(Lib.Square.wireGrpcClient)
    api(Lib.Square.okHttpClient)
    api(Lib.Kotlinx.coroutinesCore)
}