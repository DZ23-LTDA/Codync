# Your computers' network (codync-network)

The user's computers join one private network, and a bot on one messages a bot on another the way bots on one
computer already do ([bot collaboration](bot-collaboration.md)). The network is the `codync-network` crate
(`host/crates/network`, its own [README](../../host/crates/network/README.md)): peer to peer over QUIC
([iroh](https://iroh.computer)), with no Codync server involved. It is off until the user creates or joins a network.

## Set it up

From a terminal on each computer (the API methods are loopback-only: phones can't change who is in):

```sh
codync-host network create [name] [--connector public|private|https://relay]   # the first computer: the owner
codync-host network invite                                                     # prints a one-time invite (1 h)
codync-host network join '<invite>' [name] [--connector …]                     # on the other computer
codync-host network                                                            # who is around (status)
codync-host network peers | promote <computer> | remove <computer> [--cascade] | recover <code>
```

`create` prints the **recovery code** once: the network's root key, kept on no computer. The connector picks the
fallback when two computers can't connect directly: iroh's public relays (`public`, the default), the user's own
`iroh-relay` (a URL), or none (`private`: same network, Tailscale or a VPN). A computer's name defaults to its host
name; a taken name gets a number (`laptop-2`).

## Who may do what

The roster is a log of signed changes every member keeps a copy of; copies merge, and every computer works out the
same members whatever order the entries arrived in.

| Role | Can |
|---|---|
| Root key (recovery code) | name the owner (move it to another computer) |
| Owner (the computer that created the network) | invite, remove anyone, promote admins |
| Admin | invite, remove plain members |
| Member | message, list, block on itself |

A removed computer stays removed; it rejoins only with a new key. `--cascade` also voids every computer it
admitted (a stolen one). Every computer compares rosters with the others when it starts and every 6 hours, so one
that was off learns of a removal.

## Bots across computers

- Every visible bot is offered under its **address name** (`Code Reviewer` → `code-reviewer`, a second
  `reviewer` → `reviewer-2`); from another computer it is `code-reviewer@desk`.
- `list_bots` lists this computer's bots, then the other computers' (ids `name@computer`, `status` online/away),
  filtered by `query`, `computer`, `online`, and paged by `limit` (default 20, at most 50) and `cursor`. A listing
  past the limit adds `summary` (bots per computer). What other computers offer is cached for a minute and refreshed
  in the background; the first listing waits up to 5 s for it.
- `message_bot` takes a bot id, an address name, or `name@computer`. A request to another computer is a letter; the
  recipient computer turns it into a bot request in that bot's chat (its notice says "Message from Lead on laptop"),
  and its outcome goes back as a letter that names the request (`reply_to`, `meta.codync` = `completed`, `failed` or
  `cancelled`). The sender's notice ends and the sender gets the usual reply turn; a cancelled one wakes no one.
- Exchange notices carry `botMessage.peerName` ("Reviewer on desk") for the bot on the other computer, whose id
  isn't in this computer's bot list; every client shows that name in the row, the pair sheet and its author labels.
- A letter to a computer that is away waits in the sender's outbox for up to a day. The chain's hop limit counts
  across computers (replies carry it back), and the receiving computer enforces its own budgets:
  8 handoffs per chain, a burst of 5 messages per pair of agents then one every 2 minutes, the same words twice in
  10 minutes refused. A letter that isn't a request (a notice, or a reply nobody waits for after a restart) shows as
  a "Note from …" notice in the bot's chat.
- Agents that aren't Codync bots use the same letters: the standalone `codync-network` daemon (one node per
  computer without a Codync host) offers Claude Code sessions (their inbox socket) and Codex threads
  (`codex queue`), see the crate README.

## Not yet

- The apps have no network screen: setup is the CLI above.
- One network per computer; the `.network` address suffix comes with several.
- Shared networks (other people's computers) need the per-network list of offered bots and inbound rules first.
- Bots can't reach Claude Code or Codex sessions on another computer through the host (the standalone daemon can).
- The cloud assistant connector ([cloud assistants](cloud-assistants.md)) reaches only the bots of the computer it
  talks to, not that computer's network.

## Code and verification

- `host/crates/network`: the crate (`node/`, `roster.rs`, `guard.rs`, `reach.rs`, `wire.rs`); `cargo test -p codync-network`
  runs computers on one machine over direct connections, without a relay or internet.
- `host/src/network/`: the host's node, published bots and address names, the `network*` API methods.
- `host/src/chat/team/remote.rs`, `directory.rs`: requests and replies across computers, `list_bots`.
- `host/src/network_cli.rs`: `codync-host network`.
- `host/tests/network_e2e.rs`: two real hosts in one network, a bot messaging a bot on the other computer through
  approval and back.
