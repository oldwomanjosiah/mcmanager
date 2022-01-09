plugins {
    id("org.jetbrains.dokka")
}

group = "com.oldwomanjosiah"
version = "1.0"

allprojects {
    repositories {
        mavenCentral()
    }
}

tasks.dokkaHtml.configure {
    outputDirectory.set(buildDir.resolve("dokka"))
}
