package {{package}}.neoforge

import {{package}}.{{class_name}}
import net.minecraft.gametest.framework.GameTest
import net.minecraft.gametest.framework.GameTestHelper
import net.neoforged.neoforge.gametest.GameTestHolder
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate

@GameTestHolder({{class_name}}.MOD_ID)
@PrefixGameTestTemplate(false)
class {{class_name}}GameTest {
    companion object {
        @GameTest(template = "empty3x3x3")
        @JvmStatic
        fun modLoads(helper: GameTestHelper) {
            helper.succeed()
        }
    }
}
