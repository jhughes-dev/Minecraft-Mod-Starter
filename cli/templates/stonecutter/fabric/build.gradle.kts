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
    archivesName.set("$modId-fabric")
}

architectury {
    platformSetupLoomIde()
    fabric()
}

loom {
    silentMojangMappingsLicense()
    runConfigs.configureEach {
        runDir = "../../run"
    }
}

repositories {
    maven("https://maven.fabricmc.net/")
}

val common = stonecutter.node.sibling("")!!
fun dep(key: String) = common.project.property("deps.$key").toString()

configurations {
    register("common")
    register("shadowCommon")
    compileClasspath.get().extendsFrom(configurations["common"])
    runtimeClasspath.get().extendsFrom(configurations["common"])
    named("developmentFabric").get().extendsFrom(configurations["common"])
}

dependencies {
    minecraft("com.mojang:minecraft:$minecraft")
    mappings(loom.officialMojangMappings())

    modImplementation("net.fabricmc:fabric-loader:${dep("fabric_loader")}")
    modApi("net.fabricmc.fabric-api:fabric-api:${dep("fabric_api")}")

    "common"(project(path = common.project.path, configuration = "namedElements")) { isTransitive = false }
    "shadowCommon"(project(path = common.project.path, configuration = "transformProductionFabric")) { isTransitive = false }
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
        "fabric_loader" to dep("fabric_loader"),
        "mc_dep" to dep("mc_dep_fabric"),
    )
    inputs.properties(props)
    filesMatching("fabric.mod.json") { expand(props) }
}

tasks.shadowJar {
    exclude("architectury.common.json")
    configurations = listOf(project.configurations["shadowCommon"])
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

// Collect built JARs into build/libs/{mod.version}/fabric
val buildAndCollect by tasks.registering(Copy::class) {
    group = "build"
    from(tasks.remapJar.get().archiveFile)
    into(rootProject.layout.buildDirectory.file("libs/$modVersion/fabric"))
    dependsOn("build")
}
