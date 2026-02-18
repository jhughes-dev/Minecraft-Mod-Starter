package {{package}}.fabric;

import {{package}}.{{class_name}};
import net.fabricmc.api.ModInitializer;

public class {{class_name}}Fabric implements ModInitializer {
    @Override
    public void onInitialize() {
        {{class_name}}.init();
    }
}
