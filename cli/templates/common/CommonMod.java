package {{package}};

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class {{class_name}} {
    public static final String MOD_ID = "{{mod_id}}";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    public static void init() {
        LOGGER.info("Initializing {{mod_name}}");
    }
}
