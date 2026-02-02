package com.example.testmod

import org.slf4j.LoggerFactory

object TestmodMod {
    const val MOD_ID = "testmod"
    val LOGGER = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LOGGER.info("Initializing Test Mod")
    }
}
