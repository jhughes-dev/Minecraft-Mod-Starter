package {{package}}.fabric;

import {{package}}.{{class_name}};
import net.fabricmc.fabric.api.gametest.v1.FabricGameTest;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;

public class {{class_name}}GameTest implements FabricGameTest {
    @GameTest(template = EMPTY_STRUCTURE)
    public void modLoads(GameTestHelper helper) {
        helper.succeed();
    }
}
