@file:Suppress("UnstableApiUsage")

plugins {
    id("dev.architectury.loom")
    id("architectury-plugin")
{{#kotlin}}
    kotlin("jvm") version "{{kotlin_version}}"
{{/kotlin}}
}

val modId = property("mod.id").toString()
val modName = property("mod.name").toString()
val modVersion = property("mod.version").toString()
val modGroup = property("mod.group").toString()
fun dep(key: String) = property("deps.$key").toString()

val minecraft = stonecutter.current.version

version = "$modVersion+$minecraft"
group = modGroup

base {
    archivesName.set("$modId-common")
}

architectury.common(stonecutter.tree.branches.mapNotNull {
    if (stonecutter.current.project !in it) null
    else it.id.takeIf { id -> id.isNotEmpty() }
})

loom {
    silentMojangMappingsLicense()
    runConfigs.configureEach {
        runDir = "../run"
    }
}

repositories {
    maven("https://maven.fabricmc.net/")
    maven("https://maven.neoforged.net/releases/")
}

dependencies {
    minecraft("com.mojang:minecraft:$minecraft")
    mappings(loom.officialMojangMappings())
    modImplementation("net.fabricmc:fabric-loader:${dep("fabric_loader")}")
}

java {
    withSourcesJar()
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
}

tasks.withType<JavaCompile>().configureEach {
    options.encoding = "UTF-8"
    options.release.set(21)
}

{{#kotlin}}
kotlin {
    jvmToolchain(21)
}

{{/kotlin}}
// Collect built JARs into build/libs/{modVersion}/{loader}
val buildAndCollect by tasks.registering(Copy::class) {
    group = "build"
    from(tasks.remapJar.get().archiveFile)
    into(rootProject.layout.buildDirectory.file("libs/$modVersion/${stonecutter.branch.id}"))
    dependsOn("build")
}
