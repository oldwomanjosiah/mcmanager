import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlinJvm
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
    api(Lib.Square.okHttpClient)

    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<KotlinCompile>() {
    kotlinOptions.jvmTarget = "11"
}

tasks.dokkaHtml.configure {
    dependsOn(wire)

    dokkaSourceSets {
        named("main") {
            sourceRoots.builtBy(com.squareup.wire.gradle.WireTask)
            suppressGeneratedFiles.set(false)
        }
    }
}