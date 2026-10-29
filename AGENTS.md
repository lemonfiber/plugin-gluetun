# AGENTS.md — plugin-gluetun

> **Start at the roadmap and board on [lemonfiber.app](https://lemonfiber.app),
> rendered from the report of where every unreleased version stands. Then the
> rules** every repository shares:
> [working in the repositories](https://github.com/lemonfiber/spec/blob/main/50-governance/working-in-the-repositories.md)
> and [the rules for agents](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> This file holds only what is true of this repository.

## What this repo is

A bundled part: Gluetun, as the stack lemonfiber ships carries it. The
[bundle contract](https://github.com/lemonfiber/spec/blob/main/20-architecture/contracts/bundle.md)
says how the catalogue pins it and how lemonfiber carries it.

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
- **The image is Gluetun's own**, so this repository is never tagged. Moving
  the digest means recording every response again against the new image in
  the same change, committing the `proofs.json` the checks write, and reading the
  image's provider lists again for `one_of`.
