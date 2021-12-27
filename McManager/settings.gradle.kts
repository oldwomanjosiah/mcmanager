pluginManagement {
    repositories {
        google()
        gradlePluginPortal()
        maven("https://maven.pkg.jetbrains.space/public/p/compose/dev")
    }

    plugins {
        kotlin("jvm") version "1.5.31"
    }
}

rootProject.name = "McManager"

include(
    ":compose",
    ":data"
)
