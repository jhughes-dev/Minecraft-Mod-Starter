package com.example.testmod;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestmodMod {
    public static final String MOD_ID = "testmod";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    public static void init() {
        LOGGER.info("Initializing Test Mod");
    }
}
