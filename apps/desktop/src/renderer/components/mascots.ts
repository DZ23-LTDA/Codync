// The 13 mascots the host assigns (`host/src/agent/avatar.rs`) and the artwork is derived into
// `assets/mascots` by `tools/mascot-assets.mjs --derive`. Vite turns each PNG into a URL at build time.
const art = import.meta.glob<string>('../assets/mascots/*.png', { eager: true, query: '?url', import: 'default' })

/**
 * The mascot image for an id, or `undefined` when this build ships no picture for it.
 *
 * The host already resolves an unknown or unset id to one it ships (`avatar::shown`), so the
 * `undefined` here means the artwork is missing, not that the id is wrong: the caller draws the
 * procedural avatar instead and a bot still has a face.
 */
export function mascotArt(id: string | undefined): string | undefined {
  if (!id) return undefined
  return art[`../assets/mascots/${id}.png`]
}
