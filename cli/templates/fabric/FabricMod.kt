package {{package}}.fabric

import {{package}}.{{class_name}}
import net.fabricmc.api.ModInitializer

class {{class_name}}Fabric : ModInitializer {
    override fun onInitialize() {
        {{class_name}}.init()
    }
}
