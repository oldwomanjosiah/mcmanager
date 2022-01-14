plugins {
    kotlinJvm
    compose
}

dependencies {
    api(Lib.Kotlinx.coroutinesCore)

    implementation(Lib.RBusarow.dispatchCore)
    implementation(compose.runtime)
}