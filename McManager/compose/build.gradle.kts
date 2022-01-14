import org.jetbrains.compose.compose
import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlinJvm
    compose
}

group = "me.josiah"
version = "1.0"

dependencies {
    implementation(project(":data"))
    implementation(project(":base:models"))

    implementation(compose.desktop.currentOs)
    implementation(Lib.Kotlinx.coroutinesCore)
    implementation(Lib.RBusarow.dispatchCore)
}

tasks.withType<KotlinCompile>() {
    kotlinOptions.jvmTarget = "11"
}

compose.desktop {
    application {
        mainClass = "MainKt"
        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "compose"
            packageVersion = "1.0.0"
        }
    }
}
