package com.codync.android

import com.codync.android.core.VoicePhase
import com.codync.android.core.VoiceProvider
import com.codync.android.core.VoiceState
import org.junit.Assert.assertEquals
import org.junit.Test

class CallNotificationTest {
    @Test fun theNotificationShowsTheCallThenTheBotsProgress() {
        val listening = VoiceState(phase = VoicePhase.Listening, provider = VoiceProvider.Gemini)
        assertEquals("Listening · Gemini", callNotificationText(listening, null))
        assertEquals("Listening · Gemini · Running the tests", callNotificationText(listening, "Running the tests"))
        assertEquals("Speaking\nRunning the tests",
            callNotificationText(VoiceState(phase = VoicePhase.Speaking), "Running the tests", "\n"))
        assertEquals("Microphone muted", callNotificationText(listening.copy(muted = true), " "))
        assertEquals("Reconnecting…", callNotificationText(VoiceState(phase = VoicePhase.Reconnecting), null))
    }
}
