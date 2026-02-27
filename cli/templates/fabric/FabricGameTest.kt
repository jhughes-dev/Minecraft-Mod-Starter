package {{package}}.fabric

import {{package}}.{{class_name}}
import net.fabricmc.fabric.api.gametest.v1.FabricGameTest
import net.minecraft.gametest.framework.GameTest
import net.minecraft.gametest.framework.GameTestHelper

class {{class_name}}GameTest : FabricGameTest {
    @GameTest(template = FabricGameTest.EMPTY_STRUCTURE)
    fun modLoads(helper: GameTestHelper) {
        helper.succeed()
    }
}
