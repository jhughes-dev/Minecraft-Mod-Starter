import gg.meza.stonecraft.mod

plugins {
    id("gg.meza.stonecraft")
{{#kotlin}}
    kotlin("jvm")
{{/kotlin}}
}

modSettings {
    clientOptions {
        narrator = false
    }
}
