plugins {
    kotlin("jvm")
    id("org.jetbrains.dokka")
}

tasks.dokkaHtml.configure {
    outputDirectory.set(buildDir.resolve("dokka"))
}