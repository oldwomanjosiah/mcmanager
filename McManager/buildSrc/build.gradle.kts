plugins {
    kotlin("jvm") version "1.5.31"
    `kotlin-dsl`
}

repositories {
    google()
    gradlePluginPortal()
    maven("https://maven.pkg.jetbrains.space/public/p/compose/dev")
    mavenCentral()
}

dependencies {
    implementation(gradleApi())

    val kotlinVersion = "1.5.31"

    api(kotlin("gradle-plugin", version = kotlinVersion))
    api(kotlin("stdlib", version = kotlinVersion))
    api(kotlin("stdlib-common", version = kotlinVersion))
    api(kotlin("stdlib-jdk8", version = kotlinVersion))
    api(kotlin("reflect", version = kotlinVersion))

    api("com.squareup.wire:wire-gradle-plugin:4.0.1")
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    kotlinOptions {
        freeCompilerArgs += "-Xopt-in=kotlin.RequiresOptIn"
    }
}