package polyglot

import dev.drim.azimuth.Azimuth
import kotlin.test.Test
import kotlin.test.assertEquals

class IdentityServiceTest {
    @Test
    @Azimuth.Covers(
        spec = "polyglot/identity",
        scenario = "kotlin-identifies",
        scope = Azimuth.Scope.unit,
        quantification = Azimuth.Quantification.example,
        oracle = Azimuth.Oracle.direct,
    )
    fun identityIsKotlin() {
        assertEquals("kotlin", IdentityService.identity())
    }
}
