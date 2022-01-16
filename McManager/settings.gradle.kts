pluginManagement {
    repositories {
        google()
        gradlePluginPortal()
        maven("https://maven.pkg.jetbrains.space/public/p/compose/dev")
        mavenCentral()
    }

    plugins {
        kotlin("jvm") version "1.6.10"
    }
}

rootProject.name = "McManager"

include(
    ":base:data",
    ":base:models",
    ":compose",
    ":data",
)

