import com.squareup.wire.gradle.WireTask
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlinJvm
    wire
}

wire {
    kotlin {}

    sourcePath {
        val srcDir = "${rootProject.rootDir.parent}/proto"

        File(srcDir).run {
            if (!exists()) throw IllegalStateException("Protobuf src dir does not exist: $srcDir")
            if (!isDirectory) throw IllegalStateException("Protobuf src file exists but is not directory: $srcDir")
        }

        srcDir(srcDir)
    }
}

group = "me.josiah"
version = "1.0"

dependencies {
    api(project(path = ":base:data"))

    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<KotlinCompile>() {
    kotlinOptions.jvmTarget = "11"
}

tasks.dokkaHtmlPartial.configure {
    dependsOn("generateProtos")

    dokkaSourceSets {
        named("main") {
            sourceRoots.builtBy("generateProtos")
            suppressGeneratedFiles.set(false)
        }
    }
}