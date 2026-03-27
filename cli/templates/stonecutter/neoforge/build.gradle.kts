plugins {
    id("dev.architectury.loom")
    id("architectury-plugin")
    id("com.github.johnrengelman.shadow")
}

val modId = property("mod.id").toString()
val modVersion = property("mod.version").toString()
val modGroup = property("mod.group").toString()

val minecraft = stonecutter.current.version

version = "$modVersion+$minecraft"
group = modGroup

base {
    archivesName.set("$modId-neoforge")
}

architectury {
    platformSetupLoomIde()
    neoForge()
}

loom {
    silentMojangMappingsLicense()
    runConfigs.configureEach {
        runDir = "../../run"
    }
}

repositories {
    maven("https://maven.neoforged.net/releases/")
}

val common = stonecutter.node.sibling("")!!
fun dep(key: String) = common.project.property("deps.$key").toString()

configurations {
    register("common") {
        isCanBeResolved = true
        isCanBeConsumed = false
    }
    compileClasspath.get().extendsFrom(configurations["common"])
    runtimeClasspath.get().extendsFrom(configurations["common"])
    named("developmentNeoForge").get().extendsFrom(configurations["common"])

    register("shadowBundle") {
        isCanBeResolved = true
        isCanBeConsumed = false
    }
}

dependencies {
    minecraft("com.mojang:minecraft:$minecraft")
    mappings(loom.officialMojangMappings())

    "neoForge"("net.neoforged:neoforge:${dep("neoforge")}")

    "common"(project(path = common.project.path, configuration = "namedElements")) { isTransitive = false }
    "shadowBundle"(project(path = common.project.path, configuration = "transformProductionNeoForge"))
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

tasks.processResources {
    val props = mapOf(
        "version" to project.version,
        "mc_dep" to dep("mc_dep_neoforge"),
        "neoforge_dep" to dep("neoforge_dep"),
    )
    inputs.properties(props)
    filesMatching("META-INF/neoforge.mods.toml") { expand(props) }
}

tasks.shadowJar {
    exclude("architectury.common.json")
    configurations = listOf(project.configurations["shadowBundle"])
    archiveClassifier.set("dev-shadow")
}

tasks.remapJar {
    input.set(tasks.shadowJar.get().archiveFile)
    dependsOn(tasks.shadowJar)
    archiveClassifier.set(null as String?)
}

tasks.named<Jar>("sourcesJar") {
    val commonSources = project(common.project.path).tasks.named<Jar>("sourcesJar")
    dependsOn(commonSources)
    from(commonSources.map { it.archiveFile }.map { zipTree(it) })
}

components.named<AdhocComponentWithVariants>("java") {
    withVariantsFromConfiguration(configurations["shadowRuntimeElements"]) { skip() }
}

// Collect built JARs into build/libs/{mod.version}/neoforge
val buildAndCollect by tasks.registering(Copy::class) {
    group = "build"
    from(tasks.remapJar.get().archiveFile)
    into(rootProject.layout.buildDirectory.file("libs/$modVersion/neoforge"))
    dependsOn("build")
}
