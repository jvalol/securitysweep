# securitysweep

An open yard, four lights sweeping it, and the far side. The eighth game on
`blitzkit`. The dependency is the published crate, overridden by the engine
checkout at `../blitzkit` when built inside this project folder.

## Build and test

Requires Rust 1.87 or newer, the engine's MSRV.

```
cargo build
cargo test
cargo run
cargo clippy
cargo fmt
```

## How work happens here

Behavior changes are spec driven:

1. **Write the spec first.** Copy `specs/TEMPLATE.md` to `specs/NNNN-short-name.md`
   and fill it in. Say what the game does, not how the code does it.
2. **Make the acceptance criteria testable.** Each one names the test that proves
   it, or goes under "Verified by hand" when it needs a window.
3. **Write the tests, then the code.** `cargo test` passes before a commit.
4. **Update the spec when behavior changes.** A spec that disagrees with the game
   is a bug in the spec.

## Layout

- `src/main.rs` — nothing yet. The game is a spec so far.

## Why the losing condition is three engine functions

Being seen is: in the cone, in range, and nothing in the way. Those are
`SpotLight::cone`, `SpotLight::falloff`, and a ray against the cover from spec
0014. Each one also feeds the shader, so what the rule decides is what the
screen draws, and the rule can be checked without a window.

That is the whole reason to build this game. Every other game here can only be
checked by looking at it. This one's losing condition has unit tests, and they
are tests of the same arithmetic the pixels come from.

## Why four beams

blitzkit spec 0021 holds four spots and a shadow map for each. Four is the
ceiling, and it is also enough: a yard with two beams has one gap and a yard
with six has none.
