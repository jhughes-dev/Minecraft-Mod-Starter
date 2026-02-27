package {{package}};

import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class {{class_name}}Test {
    @BeforeAll
    static void setup() {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
    }

    @Test
    void modIdIsValid() {
        assertNotNull({{class_name}}.MOD_ID);
        assertTrue({{class_name}}.MOD_ID.matches("[a-z][a-z0-9_]*"));
    }
}
