import com.fasterxml.jackson.module.kotlin.kotlinModule

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