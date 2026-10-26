package com.codync.android.core

import kotlinx.serialization.json.*
import org.junit.Assert.*
import org.junit.Test

class NetworkTest {
    private val view = WireJson.parseToJsonElement("""
        {"joined": true, "computer": "desk", "role": "owner", "connector": "public", "assistants": false, "assistantsAvailable": true,
         "computers": [
           {"name": "desk", "role": "owner", "me": true, "online": true,
            "agents": [{"id": "reviewer", "name": "Reviewer"}], "canRemove": false, "canPromote": false},
           {"name": "laptop", "role": "admin", "me": false, "online": false, "agents": [],
            "canRemove": true, "canPromote": false, "later": 1}]}
    """)

    @Test fun `a network view reads the computers, their roles and what they offer`() {
        val read = NetworkView.decode(view)
        assertTrue(read.joined && read.shown && !read.assistants)
        assertEquals(listOf("desk", "laptop"), read.computers.map { it.name })
        assertEquals(listOf("Owner", "Admin"), read.computers.map { it.roleTitle })
        assertEquals(listOf("This computer", "Away"), read.computers.map { it.presence })
        assertEquals(listOf(NetworkAgent("reviewer", "Reviewer")), read.computers[0].agents)
    }

    @Test fun `a computer in no network shows only when cloud assistants can reach it and are on`() {
        fun alone(on: Boolean, available: Boolean) =
            NetworkView.decode(buildJsonObject { put("joined", false); put("assistants", on); put("assistantsAvailable", available) })
        assertFalse(alone(on = false, available = true).shown)
        assertFalse("not in an account", alone(on = true, available = false).shown)
        assertTrue(alone(on = true, available = true).shown)
    }

    @Test fun `the stream's hello carries the network, and hosts before it none`() {
        fun hello(network: JsonElement?) = MirrorEvent.decode(buildJsonObject {
            put("type", "hello"); put("hostId", "h"); put("rev", 1); network?.let { put("network", it) }
        }) as MirrorEvent.Hello
        assertEquals("desk", hello(view).network?.computer)
        assertNull(hello(null).network)
    }

    @Test fun `the network event is live state, and one this app can't read is passed over`() {
        val event = MirrorEvent.decode(buildJsonObject { put("type", "network"); put("network", view) })
        assertEquals("laptop", (event as MirrorEvent.Network).view.computers[1].name)
        val broken = buildJsonObject { put("type", "network"); putJsonObject("network") { put("computers", "many") } }
        assertEquals(MirrorEvent.Ignored, MirrorEvent.decode(broken))
        assertEquals(MirrorEvent.Ignored, MirrorEvent.decode(buildJsonObject { put("type", "network") }))
    }
}
