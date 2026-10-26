package com.codync.android.core

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement

/**
 * The computer's network of the user's computers as phones see it (`networkView`, the `network` event): read-only,
 * since who is in is decided at a computer (CodyncKit `NetworkView`).
 */
@Serializable
data class NetworkView(
    val joined: Boolean = false,
    /** This computer's name in the network. */
    val computer: String? = null,
    val assistants: Boolean = false,
    val computers: List<NetworkComputer> = emptyList(),
) {
    /** Something to show: the network, or cloud assistants turned on. */
    val shown: Boolean get() = joined || assistants

    companion object {
        fun decode(value: JsonElement): NetworkView = WireJson.decodeFromJsonElement(value)
    }
}

@Serializable
data class NetworkComputer(
    val name: String,
    /** `owner`, `admin` or `member`. */
    val role: String = "member",
    /** The computer this phone is paired with. */
    val me: Boolean = false,
    val online: Boolean = false,
    /** The bots it offers to the network. */
    val agents: List<NetworkAgent> = emptyList(),
) {
    val roleTitle: String get() = when (role) { "owner" -> "Owner"; "admin" -> "Admin"; else -> "Member" }
    val presence: String get() = when { me -> "This computer"; online -> "Online"; else -> "Away" }
}

@Serializable
data class NetworkAgent(val id: String, val name: String = id)
