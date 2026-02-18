package {{package}}

import org.slf4j.LoggerFactory

object {{class_name}} {
    const val MOD_ID = "{{mod_id}}"
    val LOGGER = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LOGGER.info("Initializing {{mod_name}}")
    }
}
