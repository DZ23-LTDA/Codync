package com.codync.android.design

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

/**
 * The mascot table is hand-kept, so the thing worth pinning is that it still covers the host's catalog and that a
 * missing id falls back instead of drawing nothing.
 */
class MascotsTest {
    /** The 13 ids in `host/src/agent/avatar.rs`, in its order. */
    private val catalog = listOf(
        "tele", "steve-jobs", "nina", "rogerio-ceni", "ed", "jo-soares", "sherlock", "flanelinha", "ju",
        "joel-santana", "pingado", "astro", "byte",
    )

    @Test
    fun everyMascotTheHostCanAssignHasAPicture() {
        assertEquals(catalog.toSet(), mascotIds())
        for (id in catalog) assertNotNull("$id has no drawable", mascotDrawable(id))
    }

    @Test
    fun anUnknownOrUnsetIdHasNoPictureSoTheDrawingStands() {
        assertNull(mascotDrawable(null))
        assertNull(mascotDrawable(""))
        assertNull(mascotDrawable("not-a-mascot"))
    }

    @Test
    fun androidForbiddenNamesAreTranslated() {
        // A hyphen and a Java keyword are both refused by the resource merger; the files are renamed to match.
        assertEquals(R.drawable.steve_jobs, mascotDrawable("steve-jobs"))
        assertEquals(R.drawable.byte_avatar, mascotDrawable("byte"))
    }
}
