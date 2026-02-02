package com.example.testmod.neoforge

import com.example.testmod.TestmodMod
import net.neoforged.bus.api.IEventBus
import net.neoforged.fml.common.Mod

@Mod(TestmodMod.MOD_ID)
class TestmodModNeoForge(modEventBus: IEventBus) {
    init {
        TestmodMod.init()
    }
}
