plugins {
    id("dev.kikugie.stonecutter")
    id("dev.architectury.loom") version "{{architectury_loom_version}}" apply false
    id("architectury-plugin") version "{{architectury_plugin_version}}" apply false
    id("com.github.johnrengelman.shadow") version "8.1.1" apply false
}

stonecutter active "{{active_version}}" /* [SC] DO NOT EDIT */

// Builds every version into build/libs/{mod.version}/{loader}
stonecutter registerChiseled tasks.register("chiseledBuild", stonecutter.chiseled) {
    group = "project"
    ofTask("buildAndCollect")
}

// Builds loader-specific versions
for (branch in stonecutter.tree.branches) {
    if (branch.id.isEmpty()) continue
    val loader = branch.id.replaceFirstChar { it.uppercase() }
    stonecutter registerChiseled tasks.register("chiseledBuild$loader", stonecutter.chiseled) {
        group = "project"
        versions { b, _ -> b == branch.id }
        ofTask("buildAndCollect")
    }
}

// Runs active version for each loader
for (node in stonecutter.tree.nodes) {
    if (node.metadata != stonecutter.current || node.branch.id.isEmpty()) continue
    val loader = node.branch.id.replaceFirstChar { it.uppercase() }
    for (type in listOf("Client", "Server")) {
        tasks.register("runActive$type$loader") {
            group = "project"
            dependsOn("${node.hierarchy}run$type")
        }
    }
}
