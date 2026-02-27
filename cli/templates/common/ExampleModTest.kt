package {{package}}

import net.minecraft.SharedConstants
import net.minecraft.server.Bootstrap
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Assertions.*

class {{class_name}}Test {
    companion object {
        @BeforeAll
        @JvmStatic
        fun setup() {
            SharedConstants.tryDetectVersion()
            Bootstrap.bootStrap()
        }
    }

    @Test
    fun modIdIsValid() {
        assertNotNull({{class_name}}.MOD_ID)
        assertTrue({{class_name}}.MOD_ID.matches(Regex("[a-z][a-z0-9_]*")))
    }
}
