package io.github.yourname.modid.neoforge;

import io.github.yourname.modid.ExampleMod;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;

@Mod(ExampleMod.MOD_ID)
public class ExampleModNeoForge {
    public ExampleModNeoForge(IEventBus modEventBus) {
        // Use modEventBus to register event listeners, e.g.:
        // modEventBus.addListener(this::onCommonSetup);
        ExampleMod.init();
    }
}
