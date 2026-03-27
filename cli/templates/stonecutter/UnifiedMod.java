package {{package}};

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
/*@Mod({{class_name}}.MOD_ID)
public class {{class_name}} {
    public {{class_name}}(IEventBus modEventBus) {
        init();
    }
*//*?} elif fabric {*/
public class {{class_name}} implements ModInitializer {
    @Override
    public void onInitialize() {
        init();
    }
/*?}*/

    public static final String MOD_ID = "{{mod_id}}";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    public static void init() {
        LOGGER.info("Initializing {{mod_name}}");
    }
}
