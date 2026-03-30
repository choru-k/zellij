<h1 align="center">
  <br>
  <img src="https://raw.githubusercontent.com/zellij-org/zellij/main/assets/logo.png" alt="logo" width="200">
  <br>
  Zellij
  <br>
  <br>
</h1>

<p align="center">
  <img src="https://raw.githubusercontent.com/zellij-org/zellij/main/assets/demo.gif" alt="demo">
</p>
<h4 align="center">
  [<a href="https://zellij.dev/documentation/installation">Installation</a>]
  [<a href="https://zellij.dev/screencasts/">Screencasts & Tutorials</a>]
  [<a href="https://zellij.dev/documentation/configuration">Configuration</a>]
  [<a href="https://zellij.dev/documentation/layouts">Layouts</a>]
  [<a href="https://zellij.dev/documentation/faq">FAQ</a>]
</h4>
<p align="center">
  <a href="https://discord.gg/CrUAFH3"><img alt="Discord Chat" src="https://img.shields.io/discord/771367133715628073?color=5865F2&label=discord&style=flat-square"></a>
  <a href="https://matrix.to/#/#zellij_general:matrix.org"><img alt="Matrix Chat" src="https://img.shields.io/matrix/zellij_general:matrix.org?color=1d7e64&label=matrix%20chat&style=flat-square&logo=matrix"></a>
  <a href="https://zellij.dev/documentation/"><img alt="Zellij documentation" src="https://img.shields.io/badge/zellij-documentation-fc0060?style=flat-square"></a>
</p>

<br>
    <p align="center">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/bc5daac4-140a-4b83-8729-71c944ee1100">
      <img src="https://github.com/user-attachments/assets/55156624-a71a-46b5-939e-f562e3b2dd7f" alt="Sponsored by ">
    </picture>
    &nbsp;
    &nbsp;
    <a href="https://www.gresearch.com/">
        <picture>
          <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/d609936a-abf8-4406-8cfc-889f76a09d74">
          <img src="https://github.com/user-attachments/assets/742ae902-fe9d-41c6-baf2-4bc143061da3" alt="gresearch logo">
        </picture>
    </a>
</p>

# What is this?

[Zellij](#origin-of-the-name) is a workspace aimed at developers, ops-oriented people and anyone who loves the terminal. Similar programs are sometimes called "Terminal Multiplexers".

Zellij is designed around the philosophy that one must not sacrifice simplicity for power, taking pride in its great experience out of the box as well as the advanced features it places at its users' fingertips.

Zellij is geared toward beginner and power users alike - allowing deep customizability, personal automation through [layouts](https://zellij.dev/documentation/layouts.html), true multiplayer collaboration, unique UX features such as floating and stacked panes, and a [plugin system](https://zellij.dev/documentation/plugins.html) allowing one to create plugins in any language that compiles to WebAssembly.

Zellij includes a built-in [web-client](https://zellij.dev/tutorials/web-client/), making a terminal optional.

You can get started by [installing](https://zellij.dev/documentation/installation.html) Zellij and checking out the [Screencasts & Tutorials](https://zellij.dev/screencasts/).

For more details about our future plans, read about upcoming features in our [roadmap](#roadmap).

## What is different in this fork?

This repository is a fork of upstream Zellij with a small set of workflow-focused additions aimed at interactive AI/agent usage, better stacked-pane ergonomics, and richer pane styling.

The main fork-specific changes so far are:

1. **Per-command synchronized-output opt-out** ([PR #1](https://github.com/choru-k/zellij/pull/1))
   - Adds `pane_synchronized_output_ignore_commands` so selected commands can bypass pane-emitted synchronized output.
   - Useful for streaming terminal apps such as `pi`, `codex`, and similar tools that otherwise look visually stalled until the next redraw.
   - Tracks the current foreground command, so it also works when those tools are started from inside a shell pane rather than launched as the pane process itself.

2. **Plugin-backed stacked pane headers** ([PR #2](https://github.com/choru-k/zellij/pull/2))
   - Adds configurable stacked pane direction, including left-aligned horizontal stacked tabs.
   - Adds a plugin-backed stacked pane header provider, allowing a background plugin to supply structured header content, styling, alignment, and actions.
   - Includes the supporting interaction model: header hit-testing, mouse actions, caching, builtin fallback behavior, and provider lifecycle fixes across tabs and reloads.

3. **Richer pane border styling** ([PR #3](https://github.com/choru-k/zellij/pull/3))
   - Replaces the old pane border color action with a more complete pane border style action that supports `--fg`, `--bg`, and `--reset`.
   - Preserves full border style data through pane, frame, and boundary rendering, including title-row and border backgrounds.
   - Fixes the live IPC delivery bug so `set-pane-border-style` works reliably in real sessions, not just in isolated code paths.

If you want the standard upstream experience, upstream Zellij remains the canonical project. This fork is for users who specifically want the workflow changes above on top of the current upstream base.

### Versioning and Homebrew naming

This fork currently tracks upstream `0.44.0` plus the fork-specific changes above.

Until the fork starts publishing tagged releases, installation examples in this README pin an explicit source revision instead of a floating fork release name. When fork releases are tagged, prefer an upstream-based version plus a fork revision, for example:
- `v0.44.0-choru.1`
- `v0.44.0-choru.2`
- `v0.45.0-choru.1`

For Homebrew, prefer a distinct formula name such as `zellij-choru` rather than reusing plain `zellij`. That makes it obvious that users are installing the fork, not upstream Zellij.

## How do I install it?
If you want the fork-specific features described above, install from this fork rather than the upstream release artifacts.

### Homebrew
This repo includes a tap-ready formula at `Formula/zellij-choru.rb`. Once the repo is tapped, you can install the fork with:
```bash
brew tap choru-k/zellij https://github.com/choru-k/zellij
brew install zellij-choru
```

This formula intentionally uses a fork-specific formula name and conflicts with upstream `zellij`, because both install the same `zellij` executable.

### Build from source
If you want to install the fork from source yourself, build it and then place the resulting binary somewhere on your `PATH`:
```bash
cargo xtask build
cp ./target/dev-opt/zellij ~/.local/bin/zellij
```

If you just want to try the fork without installing it system-wide, you can run the built binary directly:
```bash
./target/dev-opt/zellij
```

### Cargo install
If you want Cargo to install a stable fork revision, pin it to an explicit commit until fork tags exist:
```bash
cargo install --locked --git https://github.com/choru-k/zellij --rev 578fb91a096c37fb6dc249d446cda44c12c67f47 zellij
```

If you prefer to track the moving `main` branch instead, use:
```bash
cargo install --locked --git https://github.com/choru-k/zellij --branch main zellij
```

The `main` branch is pre-release code and may be broken between commits.

#### Try upstream Zellij without installing
The quick-launch script below downloads the latest upstream Zellij release from `zellij-org/zellij`. It does not include the fork-specific changes described above.

bash/zsh:
```bash
bash <(curl -L https://zellij.dev/launch)
```
fish/xonsh:
```bash
bash -c 'bash <(curl -L https://zellij.dev/launch)'
```

#### Installing from `main`
Installing Zellij from the `main` branch is not recommended. This branch represents pre-release code, is constantly being worked on and may contain broken or unusable features. In addition, using it may corrupt the cache for future versions, forcing users to clear it before they can use the officially released version.

That being said - no-one will stop you from using it (and bug reports involving new features are greatly appreciated), but please consider using a pinned revision or a tagged release once fork releases are published.

## How do I start a development environment?

* Clone the project
* In the project folder, for debug builds run: `cargo xtask run`
* To run all tests: `cargo xtask test`

For more build commands, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Configuration
For configuring Zellij, please see the [Configuration Documentation](https://zellij.dev/documentation/configuration.html).

## About issues in this repository
Issues in this repository, whether open or closed, do not necessarily indicate a problem or a bug in the software. They only indicate that the reporter wanted to communicate their experiences or thoughts to the maintainers. The Zellij maintainers do their best to go over and reply to all issue reports, but unfortunately cannot promise these will always be dealt with or even read. Your understanding is appreciated.

## Roadmap
Presented here is the project roadmap, divided into three main sections.

These are issues that are either being actively worked on or are planned for the near future.

***If you'll click on the image, you'll be led to an SVG version of it on the website where you can directly click on every issue***

[![roadmap](https://github.com/user-attachments/assets/bb55d213-4a68-4c84-ae72-7db5c9bf94fb)](https://zellij.dev/roadmap)

## Origin of the Name
[From Wikipedia, the free encyclopedia](https://en.wikipedia.org/wiki/Zellij)

Zellij (Arabic: الزليج, romanized: zillīj; also spelled zillij or zellige) is a style of mosaic tilework made from individually hand-chiseled tile pieces. The pieces were typically of different colours and fitted together to form various patterns on the basis of tessellations, most notably elaborate Islamic geometric motifs such as radiating star patterns composed of various polygons. This form of Islamic art is one of the main characteristics of architecture in the western Islamic world. It is found in the architecture of Morocco, the architecture of Algeria, early Islamic sites in Tunisia, and in the historic monuments of al-Andalus (in the Iberian Peninsula).

## License

MIT

## Sponsored by
<a href="https://terminaltrove.com/"><img src="https://avatars.githubusercontent.com/u/121595180?s=200&v=4" width="80px"></a>
