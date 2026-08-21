package polyglot

import kotlin.test.Test
import kotlin.test.assertEquals

class IdentityServiceTest {
    @Test
    fun identityIsKotlin() {
        assertEquals("kotlin", IdentityService.identity())
    }
}
