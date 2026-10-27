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
  provider, the endpoint, its key, the addresses and the private key each declare
  `recorded_with`: a documentation address, `custom`, or a value minted for the run.
  The tunnel never comes up when recorded, so its health is the control server's
  `/v1/vpn/status`, which answers whether or not the tunnel is up; whether it is up is
  read over `network.egress-guard`.
- **No Python here.** The reader is spec's `first-party-plugin` workflow's.
- **The image is Gluetun's own**, so this repository is never tagged. Moving
  the digest means recording every response again against the new image in
  the same change, and committing the `proofs.json` the checks write.
