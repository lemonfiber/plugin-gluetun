# AGENTS.md — plugin-gluetun

> **Start at the roadmap and board on [lemonfiber.app](https://lemonfiber.app),
> rendered from the report of where every unreleased version stands. Then the
> rules** every repository shares:
> [working in the repositories](https://github.com/lemonfiber/spec/blob/main/50-governance/working-in-the-repositories.md)
> and [the rules for agents](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> This file holds only what is true of this repository.

## What this repo is

Gluetun as a first-party lemonfiber plugin: `plugin.toml` and the adapter that
speaks the `network.egress-guard@1` contract in front of Gluetun. `up` and
`forwarded` read Gluetun's control server under the key lemonfiber writes into
the adapter's `upstream.json`, and `secure` writes the role file Gluetun reads
when it starts, holding its control server to that key. The adapter is the Rust
crate at the root, built on `lemonfiber-adapter-kit` from the core at the commit
`Cargo.toml` pins, into the image the `Dockerfile` builds. `release.yml`
publishes that image from every commit on `main` and from the train's tag.
`plugin.toml` runs Gluetun alone, and its claim's recordings are Gluetun's own
answers. The
[bundle contract](https://github.com/lemonfiber/spec/blob/main/20-architecture/contracts/bundle.md)
says how the catalogue pins this repository and how lemonfiber carries it.

## The rules you cannot break

- **The plugin id is the service's id** (`ARCH-R266`). The container, the
  configuration directory, the settings and the journal keep their names.
- **One service, speaking no contract**, while the bundle assembles it. It
  declares `replaces` naming its own id, which is how it takes the id, the port
  and the hostname the bundle reserves for it (`ARCH-R222`).
- **Its claims' recordings sit under `recordings/gluetun/`**, and its starting
  files, where it has any, under `config/gluetun/` (`ARCH-R271`).
- **Every service declares `reaches` and `asks_for`** (`ARCH-R270`).
- **What only the operator holds is stood in for when recorded** (`ARCH-R336`). The
  VPN type, the provider, the endpoint, its key, the addresses, the private and
  pre-shared keys and the OpenVPN username and password each declare `recorded_with`:
  `wireguard`, a documentation address, `custom`, or a value minted for the run. A
  setting without one is not written when recorded.
  The tunnel never comes up when recorded, so its health is the control server's
  `/v1/vpn/status`, which answers whether or not the tunnel is up; whether it is up is
  read over `network.egress-guard`.
- **Its settings name the values Gluetun accepts** (`ARCH-R348`, ADR-0055). The VPN
  type is `wireguard` or `openvpn`; each provider is offered only over the type Gluetun
  runs it over with no more than a key, an address, a username and a password; port
  forwarding is offered only with a provider that forwards one; and each variable and
  secret is written only beside the values that need it (`ARCH-R349`). A missing one
  holds the guard back by name rather than leaving it restarting (`ARCH-R351`).
- **No Python here.** The reader is spec's `first-party-plugin` workflow's.
- **No `unwrap`, `expect`, `panic` or indexing** outside tests. The lints in
  `Cargo.toml` are the core's.
- **The role file opens two routes to the key and nothing else.** One role,
  `apikey`, for `GET /v1/vpn/status` and `GET /v1/portforward`; every other route
  of the control server, and those two without the key, are refused. It is
  written owner-only (`0600`) through `lemonfiber_adapter_kit::shared`, so a link,
  a hardlink or a file that is not plain is replaced and never written through,
  and a role file that is not the one the key makes is written over, since one
  changed by hand can only open more.
- **The tunnel is up only where Gluetun says it runs and its own health check
  passes.** The health server, at `HEALTH`, is asked with nothing and only its
  status is read; what it says on a failure is never passed on.
- **`secure` answers at once only where Gluetun proves it holds the key**: the file
  is the one the key makes, Gluetun admits a request under the key and refuses one
  without it. Anything else is taken up when Gluetun next starts.
- **The core's crates move with the train.** The `rev` in `Cargo.toml` is the
  core commit `targets.toml` names, the same for every `lemonfiber-*` crate.
- **Moving Gluetun's digest** means recording every response again against the
  new image in the same change, moving `RELEASE` and `DIGEST` in `src/adapter.rs`
  with it, committing the `proofs.json` the checks write, and reading the image's
  provider lists again for `one_of`.
