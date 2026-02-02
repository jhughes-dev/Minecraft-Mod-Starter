package com.example.testmod.fabric

import com.example.testmod.TestmodMod
import net.fabricmc.api.ModInitializer

class TestmodModFabric : ModInitializer {
    override fun onInitialize() {
        TestmodMod.init()
    }
}
