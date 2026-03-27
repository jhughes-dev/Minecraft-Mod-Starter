package com.example.testmod

import org.slf4j.LoggerFactory

/*? if fabric {*/
import net.fabricmc.api.ModInitializer
/*?}*/

/*? if neoforge {*/
/*import net.neoforged.bus.api.IEventBus
import net.neoforged.fml.common.Mod
*//*?}*/

/*? if neoforge {*/
/*@Mod(TestmodMod.MOD_ID)
class TestmodMod(modEventBus: IEventBus) {
    init {
        Companion.init()
    }
*//*?} elif fabric {*/
class TestmodMod : ModInitializer {
    override fun onInitialize() {
        init()
    }
/*?}*/

    companion object {
        const val MOD_ID = "testmod"
        val LOGGER = LoggerFactory.getLogger(MOD_ID)

        fun init() {
            LOGGER.info("Initializing Test Mod")
        }
    }
}
