package {{package}}.neoforge

import {{package}}.{{class_name}}
import net.neoforged.bus.api.IEventBus
import net.neoforged.fml.common.Mod

@Mod({{class_name}}.MOD_ID)
class {{class_name}}NeoForge(modEventBus: IEventBus) {
    init {
        {{class_name}}.init()
    }
}
