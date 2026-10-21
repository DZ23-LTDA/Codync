import { useEffect, useRef, useState } from 'react'
import type { SpeechEvent } from '@shared/ipc'
import { VoiceKeyRouter, type VoiceKey } from '@shared/voice-shortcut'
import { Observable, useModel } from '../../lib/observable'
import { VoiceSettings } from '../call/voice'

/** The running dictation: one recognizer at a time, and a call stops dictation first. */
let active: Dictation | null = null

export function stopDictation() {
  active?.cancel()
}

/** `said` after what's already in the box, with one space between. */
export function joinDictated(before: string, said: string) {
  if (!before.trim()) return said
  if (!said) return before
  return /\s$/.test(before) ? before + said : `${before} ${said}`
}

/**
 * Dictation with on-device speech (the speech helper; macOS). Words reach `write` as they're
 * recognized, after what was already typed. The helper doesn't stop on a pause: it ends on
 * `finish()` once the last words are in, on `cancel()`, on an error, or when another window takes it.
 */
export class Dictation extends Observable {
  readonly available = window.codync.speech.available
  listening = false
  /** When it started listening (ms since the epoch). */
  startedAt = 0
  /** The microphone's level while listening, 0…1. Read it each frame: it doesn't re-render. */
  level = 0
  /** Why the last dictation ended early (no microphone, speech recognition not allowed), until the next. */
  error: string | null = null
  /** The box up to the words still being recognized. */
  private before = ''
  private off: (() => void) | null = null
  private done: (() => void) | null = null
  /** Started by the voice input shortcut's double tap: its next tap sends what's in the field. */
  private byShortcut = false
  private readonly keys = new VoiceKeyRouter()

  constructor(private write: (text: string) => void) {
    super()
  }

  /** Listening, or stopped and waiting for the last words. */
  get busy() {
    return this.off !== null
  }

  start(draft: string, byShortcut = false) {
    active?.cancel()
    active = this
    this.byShortcut = byShortcut
    this.error = null
    this.before = draft
    this.off = window.codync.speech.onEvent((e) => this.event(e))
    window.codync.speech.start(VoiceSettings.language() || null)
    this.startedAt = Date.now()
    this.listening = true
    this.changed()
  }

  /** Stops listening; `done` runs once the recognizer has delivered its last words. */
  finish(done?: () => void) {
    this.byShortcut = false
    if (!this.off) return done?.()
    this.done = done ?? null
    this.listening = false
    this.level = 0
    this.changed()
    window.codync.speech.stop()
  }

  /** Stops listening now and drops words still on their way. */
  cancel() {
    this.byShortcut = false
    this.error = null
    if (!this.off) return
    window.codync.speech.stop()
    this.end()
  }

  /**
   * A tap of the voice input shortcut: a tap after its double tap finishes and runs `send`. True
   * when it's a double tap that should start: the caller shows its field, then `start(draft, true)`.
   */
  shortcutKey(key: VoiceKey, send: () => void): boolean {
    const action = this.keys.route(key, { byShortcut: this.byShortcut, busy: this.busy })
    if (action === 'send') this.finish(send)
    return action === 'start'
  }

  private event(e: SpeechEvent) {
    if (e.type === 'level') this.level = e.value
    if (e.type === 'partial' || e.type === 'final') this.write(joinDictated(this.before, e.text))
    if (e.type === 'final') this.before = joinDictated(this.before, e.text)
    if (e.type === 'error') {
      // Nothing to send, and the helper (which may still hold the microphone) goes too.
      this.byShortcut = false
      this.error = e.message
      window.codync.speech.stop()
    }
    if (e.type === 'stopped' || e.type === 'error') this.end()?.()
  }

  /** Returns what was waiting for the last words. */
  private end() {
    this.off?.()
    this.off = null
    this.listening = false
    this.level = 0
    if (active === this) active = null
    const done = this.done
    this.done = null
    this.changed()
    return done
  }
}

/** A composer's dictation; it stops when the composer goes away (another chat). */
export function useDictation(setDraft: (text: string) => void) {
  const write = useRef(setDraft)
  write.current = setDraft
  const [dictation] = useState(() => new Dictation((text) => write.current(text)))
  useEffect(() => () => dictation.cancel(), [dictation])
  return useModel(dictation)
}
