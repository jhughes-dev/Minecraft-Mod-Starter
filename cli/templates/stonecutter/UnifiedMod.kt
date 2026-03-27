package {{package}}

import org.slf4j.LoggerFactory

/*? if fabric {*/
import net.fabricmc.api.ModInitializer
/*?}*/

/*? if neoforge {*/
/*import net.neoforged.bus.api.IEventBus
import net.neoforged.fml.common.Mod
*//*?}*/

/*? if neoforge {*/
/*@Mod({{class_name}}.MOD_ID)
class {{class_name}}(modEventBus: IEventBus) {
    init {
        Companion.init()
    }
*//*?} elif fabric {*/
class {{class_name}} : ModInitializer {
    override fun onInitialize() {
        init()
    }
/*?}*/

    companion object {
        const val MOD_ID = "{{mod_id}}"
        val LOGGER = LoggerFactory.getLogger(MOD_ID)

        fun init() {
            LOGGER.info("Initializing {{mod_name}}")
        }
    }
}
