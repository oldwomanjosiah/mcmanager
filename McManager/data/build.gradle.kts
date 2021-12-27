import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm")
    wire
}

wire {
    kotlin {
        rpcRole = "client"
    }
}

group = "me.josiah"
version = "1.0"

dependencies {
    api(Lib.Square.wireRuntime)
    api(Lib.Square.wireGrpcClient)

    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<KotlinCompile>() {
    kotlinOptions.jvmTarget = "11"
}