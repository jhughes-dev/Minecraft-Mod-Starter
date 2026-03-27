package com.example.testmod;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/*? if fabric {*/
import net.fabricmc.api.ModInitializer;
/*?}*/

/*? if neoforge {*/
/*import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
*//*?}*/

/*? if neoforge {*/
/*@Mod(TestmodMod.MOD_ID)
public class TestmodMod {
    public TestmodMod(IEventBus modEventBus) {
        init();
    }
*//*?} elif fabric {*/
public class TestmodMod implements ModInitializer {
    @Override
    public void onInitialize() {
        init();
    }
/*?}*/

    public static final String MOD_ID = "testmod";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    public static void init() {
        LOGGER.info("Initializing Test Mod");
    }
}
