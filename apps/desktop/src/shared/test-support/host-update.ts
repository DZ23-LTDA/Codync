import { EventEmitter } from 'node:events'
import * as crypto from 'node:crypto'
import * as path from 'node:path'
import { environmentProfile, hostEnvironmentError } from '../environment.ts'
import { loadMainModule, settle } from './main-module.ts'

export function hostUpdateFixture(options: { environment?: 'main' | 'dev'; platform?: string; devPort?: string; installed?: boolean; missingBinary?: boolean; restartMarker?: boolean } = {}) {
  const identity = environmentProfile(options.environment ?? 'main')
  const platform = options.platform ?? 'darwin'
  // The fixture simulates the platform it names, not the one running the test: a
  // Windows checkout must exercise the same darwin and linux paths a Mac runs. So
  // the module gets that platform's path rules and that platform's own path shapes.
  const paths = platform === 'win32' ? path.win32 : path.posix
  const home = platform === 'win32' ? 'C:\\Users\\test' : '/home/test'
  const resources = platform === 'win32' ? 'C:\\bundle' : '/bundle'
  const prefsDir = platform === 'win32' ? 'C:\\prefs' : '/prefs'
  const mainDir = platform === 'win32' ? 'C:\\app\\out\\main' : '/app/out/main'
  const binary = platform === 'win32' ? `${resources}\\codync-host.exe` : platform === 'linux' ? `${home}/.local/bin/codync-host` : `${resources}/codync-host`
  const binaryBytes = Buffer.from('new signed host fixture')
  const persisted: Record<string, unknown> = { hostRestartAfterAppUpdate: options.restartMarker ?? false }
  const calls: string[][] = []
  const order: string[] = []
  const fetched: string[] = []
  const timers = new Map<number, { callback: () => void; ms: number }>()
  let timerId = 0
  const behavior = {
    stopStatus: 0,
    installStatus: 0,
    installed: options.installed ?? true,
    busy: false,
    updateRequested: null as number | null,
    reachable: true,
    wrongBinary: false,
    binaryEnvironment: identity.environment as string,
    runningEnvironment: identity.environment as string,
  }
  const service = () => {
    if (!behavior.installed) throw new Error('No service')
    return binary.replace(/codync-host\.exe$/, identity.supervisor)
  }
  const fs = {
    existsSync: (name: string) => !options.missingBinary && name === binary,
    realpathSync: (name: string) => name,
    readFileSync: (name: string) => name.endsWith('host-prefs.json') ? JSON.stringify(persisted) : service(),
    mkdirSync: () => {},
    writeFileSync: (_name: string, value: string) => { Object.assign(persisted, JSON.parse(value)) },
    promises: {
      readFile: async (name: string) => /[\\/]token$/.test(name) ? 'fixture-token' : binaryBytes,
    },
  }
  const module = loadMainModule<typeof import('../../main/host-controller.ts')>('host-controller', {
    'node:events': { EventEmitter }, 'node:crypto': crypto, 'node:path': paths, 'node:fs': fs,
    'node:os': { homedir: () => home, platform: () => platform },
    'node:child_process': {
      execFileSync: () => service(),
      execFile: (_bin: string, args: string[], _options: unknown, callback: (error: { code: number } | null, stdout: string, stderr: string) => void) => {
        if (args[0] === 'compat') {
          callback(null, JSON.stringify({ environment: behavior.binaryEnvironment }), '')
          return
        }
        calls.push([...args]); order.push(args[0]!)
        const status = args[0] === 'stop' ? behavior.stopStatus : behavior.installStatus
        callback(status ? { code: status } : null, '', status ? 'service command failed' : '')
      },
    },
    electron: { app: { isPackaged: true, getPath: () => prefsDir } },
    './environment': { identity },
    '../shared/environment': { hostEnvironmentError },
    './screen': { unregisterScreenAgent: () => { order.push('unregisterScreen') } },
  }, {
    __dirname: mainDir,
    process: { env: options.devPort ? { CODYNC_PORT: options.devPort } : {}, resourcesPath: resources },
    setTimeout: (callback: () => void, ms: number) => { const id = ++timerId; timers.set(id, { callback, ms }); return id },
    clearTimeout: (id: number) => timers.delete(id),
    fetch: async (url: string) => {
      fetched.push(url)
      if (!behavior.reachable) throw new Error('Host offline')
      return { ok: true, json: async () => ({ environment: behavior.runningEnvironment, computerId: 'fixture-computer', busy: behavior.busy, updateRequested: behavior.updateRequested, binaryPath: binary, binaryHash: behavior.wrongBinary ? 'stale' : crypto.createHash('sha256').update(binaryBytes).digest('hex') }) }
    },
  })
  const host = new module.HostController()
  const tick = async (ms: number) => {
    for (const [id, timer] of [...timers]) {
      if (timer.ms !== ms) continue
      timers.delete(id)
      timer.callback()
    }
    await settle()
  }
  return { host, prefs: module.prefs, persisted, calls, order, fetched, behavior, tick }
}
