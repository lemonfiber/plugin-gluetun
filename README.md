# Gluetun for lemonfiber

**Routes torrent traffic through your VPN and blocks it if the VPN drops.**
Gluetun is one of the parts the stack lemonfiber ships is built from.
Without it: Your home IP is visible to every peer.

## How lemonfiber carries it

The default bundle in
[lemonfiber-plugins](https://github.com/lemonfiber/lemonfiber-plugins) pins this
repository by a revision and the digests of its manifest and its tree, and
lemonfiber compiles that tree in.
lemonfiber assembles Gluetun into the stack it ships as a stack service.
Its image is Gluetun's own, so the release train never tags this repository, and
its pin moves by a reviewed pull request on the catalogue.

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The part's manifest |
| `recordings/gluetun/` | The responses its claims are proved against |
| `targets.toml` | The lemonfiber its checks run against |
| `proofs.json` | What those checks said, committed |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
claim against its recording, and every pinned digest. This repository holds no
Python of its own. `just ci` runs the gates that need nothing but this tree.

## Licence

The data in this repository is under the licence in `LICENSE`.
Gluetun itself is under MIT and is not distributed here: this
repository names an image, it does not contain one.
