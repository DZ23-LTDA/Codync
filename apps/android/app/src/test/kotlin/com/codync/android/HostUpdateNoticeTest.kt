package com.codync.android

import com.codync.android.core.VersionMismatch
import org.junit.Assert.*
import org.junit.Test

class HostUpdateNoticeTest {
    private val behind = VersionMismatch.UpdateHost("2.23.1", "2.26.0")
    private val needs = "Mini runs Codync 2.23.1; this app needs 2.26.0 or newer."
    private val now = 1_000_000_000L
    private fun notice(canUpdate: Boolean, update: HostUpdate?, os: String? = "macos") = hostUpdateNotice("Mini", behind, os, canUpdate, update, now)

    @Test fun aComputerThatCanUpdateOffersUpdateNow() {
        assertEquals(HostUpdateNotice(needs, true), notice(true, null))
    }

    @Test fun aRequestedUpdateShowsAsUnderWayUntilItsWaitIsOver() {
        assertEquals(HostUpdateNotice("Updating Codync on Mini. This takes a few minutes.", false),
            notice(true, HostUpdate.Requested(now - HOST_UPDATE_WAIT_MS + 1)))
        assertEquals(HostUpdateNotice("$needs Mini didn't finish updating. Try again, or update Codync on that computer.", true),
            notice(true, HostUpdate.Requested(now - HOST_UPDATE_WAIT_MS)))
    }

    @Test fun aRefusedUpdateSaysWhy() {
        val busy = "A bot is working on this computer. Update it once the bot is done."
        assertEquals(HostUpdateNotice("$needs $busy", true), notice(true, HostUpdate.Failed(busy)))
    }

    @Test fun anOlderHostKeepsItsInstructions() {
        assertEquals(HostUpdateNotice("$needs Open Codync on that Mac and choose Check for Updates.", false), notice(false, null))
        assertEquals(HostUpdateNotice("$needs Run codync-host update there.", false), notice(false, null, "linux"))
    }

    // The Codync row on the computer's card in Computer management (iOS `HostVersionLine`).
    private fun line(latest: String?, mismatch: VersionMismatch? = null, canUpdate: Boolean = true, update: HostUpdate? = null) =
        hostVersionLine("2.28.0", latest, mismatch, canUpdate, update, "Mini", "macos", now)

    @Test fun theCodyncRowOffersANewerReleaseOrSaysItsCurrent() {
        assertEquals(HostVersionLine("2.28.0", listOf("Up to date")), line("2.28.0"))
        assertEquals(listOf("Up to date"), line("2.27.0").detail)
        // A computer that doesn't know the latest release makes no claim.
        assertEquals(HostVersionLine("2.28.0", emptyList()), line(null))
        assertEquals(HostVersionLine("2.28.0", listOf("2.29.0 available"), action = "Update"), line("2.29.0"))
        assertEquals(HostVersionLine("2.28.0", listOf("2.29.0 available", "Open Codync on that Mac and choose Check for Updates.")),
            line("2.29.0", canUpdate = false))
        assertEquals(HostVersionLine("2.28.0", listOf("Updating Codync on Mini. This takes a few minutes."), updating = true),
            line("2.29.0", update = HostUpdate.Requested(now)))
    }

    @Test fun aComputerTooOldOrAnUpdateThatDidntTakeIsAProblem() {
        assertEquals(HostVersionLine("2.28.0", listOf("This app needs 2.26.0 or newer."), warns = true, action = "Update"), line(null, behind))
        val busy = "A bot is working on this computer. Update it once the bot is done."
        assertEquals(HostVersionLine("2.28.0", listOf(busy), warns = true, action = "Try again"), line("2.29.0", update = HostUpdate.Failed(busy)))
    }
}
