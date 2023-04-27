# gbemu-rs - A Game Boy emulator written in Rust for fun and learning

gbemu-rs is a toy Game Boy emulator written in Rust with the purpose of
learning about writing emulators and Rust at the same time. It's pretty
rudimentary but it can run (at least) Tetris and Super Mario Land.

[gbemu-rs.webm](https://user-images.githubusercontent.com/507354/234896287-3c652001-5f1a-4fe6-83fd-2e62346e84a9.webm)

## Getting started

1. Clone the repo

```console
git clone https://github.com/iaguis/gbemu-rs
```

2. Build

```console
cargo build --release
```

3. Run

```
./target/release/gbemu-rs $GAME_BOY_ROM
```

## Keybindings

| Key                 | Game Boy button    |
| ------------------- | ------------------ |
|  A                  | B                  |
|  S                  | A                  |
|  G                  | Start              |
|  H                  | Select             |
|  Up/Down/Left/Right | Up/Down/Left/Right |

## Implemented

* CPU
    * All instructions correct (Passes Blargg `cpu_instrs` tests)
    * Timings are not accurate
* GPU
    * Background
    * 8x8 sprites
* Keypad
* Timer
* MMU
    * Games with no MBC (e.g. Tetris)
    * Games with MBC1 (e.g. Super Mario Land)
* Serial
* Rudimentary debugger (gbdb)

## TODO

* Refactor and clean up the code
* Audio
* GPU
    * Window support
    * 8x16 sprite support
* Savegames
* MBC2+
* Game Boy Color support
* Fix some [bugs](BUGS.md)

## References

* https://media.ccc.de/v/33c3-8029-the_ultimate_game_boy_talk
* https://imrannazar.com/GameBoy-Emulation-in-JavaScript:-The-CPU
* https://github.com/mvdnes/rboy/
* https://gbdev.io/pandocs/
* https://gbdev.gg8.se/files/roms/blargg-gb-tests/
