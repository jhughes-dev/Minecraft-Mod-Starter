package com.example.testmod.fabric;

import com.example.testmod.TestmodMod;
import net.fabricmc.api.ModInitializer;

public class TestmodModFabric implements ModInitializer {
    @Override
    public void onInitialize() {
        TestmodMod.init();
    }
}
