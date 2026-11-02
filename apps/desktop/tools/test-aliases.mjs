// Lets `node --test` load renderer modules that import through the app's path aliases.
//
// The bundler and `tsc` resolve `@shared/*` from `tsconfig.json`; node's ESM resolver has never heard of it, so any
// store that imports `@shared/models` was impossible to test until this hook existed. It maps the aliases onto real
// files (with the `.ts` extension `--experimental-strip-types` expects) and leaves every other specifier alone.
//
// Loaded by the `test` script: `node --test --experimental-strip-types --import ./tools/test-aliases.mjs`.
import { existsSync } from 'node:fs'
import { registerHooks } from 'node:module'
import { dirname, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
/** Alias prefix → the folder under `apps/desktop` it stands for (see `tsconfig.json`). */
const ALIASES = { '@shared/': 'src/shared/', '@main/': 'src/main/' }

registerHooks({
  resolve(specifier, context, next) {
    for (const [prefix, folder] of Object.entries(ALIASES)) {
      if (!specifier.startsWith(prefix)) continue
      const name = specifier.slice(prefix.length)
      for (const candidate of [`${folder}${name}.ts`, `${folder}${name}/index.ts`]) {
        const path = resolve(root, candidate)
        if (existsSync(path)) return { url: pathToFileURL(path).href, shortCircuit: true }
      }
    }
    return next(specifier, context)
  },
})
