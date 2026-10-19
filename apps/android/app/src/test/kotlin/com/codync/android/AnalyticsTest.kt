package com.codync.android

import org.junit.Assert.assertEquals
import org.junit.Test

class AnalyticsTest {
    @Test fun `the computer tag matches the host's hash`() {
        // sha256("codync-analytics:<id>")[0..8], as host/src/analytics.rs and the iPhone compute it.
        assertEquals("fbfcd4f4fe9f876c", Analytics.computerTag("tZ1WMzsVltt-fj7mUDwZsg"))
    }

    @Test fun `a crash keeps its type and Codync frames but never its message`() {
        val error = IllegalStateException("secret prompt text")
        val properties = CrashReports.properties(error).toString()
        assert("IllegalStateException" in properties)
        assert("secret prompt text" !in properties)
    }

    @Test fun `a session ends after half an hour idle or a day in all`() {
        val session = AnalyticsSession()
        val t0 = 1_800_000_000_000L
        val minute = 60_000L
        val first = session.id(t0)
        // A UUIDv7 whose first 48 bits are its start in milliseconds.
        assertEquals("%012x".format(t0) + "7", first.replace("-", "").take(13))
        assertEquals(first, session.id(t0 + 29 * minute))
        val start = t0 + 60 * minute
        val second = session.id(start)
        assert(second != first) { "idle for 31 minutes" }
        // Busy without a break: a new one once it has run for a day.
        var t = start
        while (t <= start + 24 * 60 * minute) { assertEquals(second, session.id(t)); t += 20 * minute }
        assert(session.id(t) != second)
    }
}
