package com.codync.android

import com.codync.android.core.*
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.update

// The computer's network as this phone sees it: read-only, fetched when the link is up, kept current by its event.

/** `networkView` once the link is up; a host that predates it (or can't answer) leaves it unset, so nothing shows. */
internal suspend fun AppStore.loadNetwork(active: EncryptedChannel) {
    val view = try { NetworkView.decode(active.call("networkView")) }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (_: Exception) { null }
    mutableState.update { it.copy(network = view) }
}

internal fun AppStore.receiveNetwork(events: List<MirrorEvent>) {
    val view = events.filterIsInstance<MirrorEvent.Network>().lastOrNull()?.view ?: return
    mutableState.update { it.copy(network = view) }
}
