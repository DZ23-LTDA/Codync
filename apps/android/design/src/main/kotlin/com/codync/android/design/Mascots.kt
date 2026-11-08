package com.codync.android.design

import androidx.annotation.DrawableRes

/**
 * The mascot art for the ids the host resolves a bot to (`host/src/agent/avatar.rs`).
 *
 * The pictures are derived by `tools/mascot-assets.mjs --derive`, which also owns the file names: Android refuses a
 * hyphen and a Java keyword, so `steve-jobs` ships as `steve_jobs` and `byte` as `byte_avatar`. This map is the one
 * place that translation is written down; [mascotDrawable] returning null is why a bot falls back to the drawn avatar
 * instead of showing nothing.
 */
private val MASCOTS: Map<String, Int> = mapOf(
    "tele" to R.drawable.tele,
    "steve-jobs" to R.drawable.steve_jobs,
    "nina" to R.drawable.nina,
    "rogerio-ceni" to R.drawable.rogerio_ceni,
    "ed" to R.drawable.ed,
    "jo-soares" to R.drawable.jo_soares,
    "sherlock" to R.drawable.sherlock,
    "flanelinha" to R.drawable.flanelinha,
    "ju" to R.drawable.ju,
    "joel-santana" to R.drawable.joel_santana,
    "pingado" to R.drawable.pingado,
    "astro" to R.drawable.astro,
    "byte" to R.drawable.byte_avatar,
)

/** The picture for a mascot id, or null when this build ships none for it. */
@DrawableRes
fun mascotDrawable(id: String?): Int? = id?.let { MASCOTS[it] }

/** Every mascot id this build can draw, for the drift test that compares it with the host's catalog. */
fun mascotIds(): Set<String> = MASCOTS.keys
