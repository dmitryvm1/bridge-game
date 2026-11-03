# Bridge

A native Rust desktop card game using **GPUI Community Edition (`gpui-ce`)**. Play the 36-card game in [DESIGN.md](DESIGN.md) against one, two, or three computer opponents.

The interface includes a green card table, same-rank group selection, Jack suit selection, optional Queen of Spades penalties, a move history, round results, and cumulative scores. Computer turns advance automatically with a short pause so moves remain readable. Scores are kept for the current game in memory.

## Run

Requires Rust **1.95+**, network access for the initial Cargo dependency download, and a graphical desktop with a GPU supported by GPUI.

```sh
cargo run --release
```

- **Linux:** X11 or Wayland and Vulkan drivers. On Ubuntu/Debian, install the native development dependencies below.
- **macOS:** Xcode Command Line Tools (`xcode-select --install`); GPUI uses the native Metal backend.
- **Windows:** Rust's MSVC toolchain, Visual Studio C++ Build Tools, and the Windows SDK.

```sh
sudo apt-get install build-essential pkg-config cmake ninja-build clang libclang-dev \
  libasound2-dev libfontconfig1-dev libfreetype6-dev libx11-xcb-dev \
  libxkbcommon-x11-dev libwayland-dev libvulkan-dev libxcb-xinput-dev \
  libxcb-xkb-dev libxcb-randr0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

For backend setup details, see the [GPUI project](https://github.com/gpui-ce/gpui-ce).

## Play

Choose 2–4 total players and the Queen of Spades option, then take your seat. Click a legal card to select it. You can add other cards of the same rank, in the order you want to play them; the first card must match the current suit or rank. Click **Play selected**. Choose a suit before playing Jacks. Click **Draw cards** when you have no legal move; sixes automatically draw until covered or the available cards run out.

Seven penalties are accepted immediately; the recipient then takes a normal turn. Eights and Aces distribute their skips among the other players, excluding the actor. The dealer's opening card also applies its effect. An opening Jack uses its printed suit. A winning play applies its card effects before the round is scored, including mandatory draws for the other players. Dealers rotate clockwise. Exactly 125 resets to zero, and a score above 125 ends the game.

The design does not specify a fully blocked table (all drawable cards exhausted, no legal move in any hand). To ensure progress, the implementation settles that round in favor of the lowest hand penalty, breaking ties by seating order. If multiple players exceed 125 simultaneously, the first in seating order is reported as the loser; all scores remain visible.

**New game** returns to setup and abandons the current game. **How to play** displays the original rules.

## Checks

```sh
cargo fmt --check
cargo test --lib
cargo check --all-targets
```

The game engine has no dependencies and can also be tested without downloading GPUI:

```sh
rustc --edition 2024 --test src/game.rs -o /tmp/bridge-game-tests
/tmp/bridge-game-tests
```

CI is configured to test and check the desktop app on Linux, macOS, and Windows. The initial implementation's engine tests passed locally; the desktop build and visual inspection could not be performed in the authoring sandbox because crates.io DNS was unavailable. The cross-platform workflow must pass before treating the desktop build as verified.

## Structure

- `src/game.rs`: rules, dealing, penalties, scoring, computer player, and engine tests.
- `src/main.rs`: GPUI desktop application and card table.
- `src/lib.rs`: engine module export for library tests.
