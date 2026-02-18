package {{package}}.neoforge;

import {{package}}.{{class_name}};
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;

@Mod({{class_name}}.MOD_ID)
public class {{class_name}}NeoForge {
    public {{class_name}}NeoForge(IEventBus modEventBus) {
        {{class_name}}.init();
    }
}
