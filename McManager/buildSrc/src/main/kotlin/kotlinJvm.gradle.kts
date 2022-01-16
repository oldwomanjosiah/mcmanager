import org.jetbrains.dokka.gradle.DokkaTaskPartial

plugins {
    kotlin("jvm")
    id("org.jetbrains.dokka")
}

tasks.withType<DokkaTaskPartial>().configureEach {
    outputDirectory.set(buildDir.resolve("dokka"))
    moduleName.set(project.path)
}