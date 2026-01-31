package io.github.yourname.modid.neoforge;

import io.github.yourname.modid.ExampleMod;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;

@Mod(ExampleMod.MOD_ID)
public class ExampleModNeoForge {
    public ExampleModNeoForge(IEventBus modEventBus) {
        ExampleMod.init();
    }
}
