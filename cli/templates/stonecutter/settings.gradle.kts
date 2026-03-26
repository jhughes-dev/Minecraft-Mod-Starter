pluginManagement {
    repositories {
        mavenCentral()
        gradlePluginPortal()
        maven("https://maven.fabricmc.net/")
        maven("https://maven.architectury.dev/")
        maven("https://maven.neoforged.net/releases/")
        maven("https://maven.kikugie.dev/snapshots")
    }
}

plugins {
    id("dev.kikugie.stonecutter") version "0.6"
}

stonecutter {
    centralScript = "build.gradle.kts"
    kotlinController = true
    create(rootProject) {
        versions({{stonecutter_versions}})
{{#fabric}}
        branch("fabric")
{{/fabric}}
{{#neoforge}}
        branch("neoforge")
{{/neoforge}}
    }
}

rootProject.name = "{{mod_name}}"
