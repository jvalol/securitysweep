# 0003 A picture of the yard from above

**Status:** implemented
**Date:** 2026-11-03

## Goal

The frame securitysweep puts on its readme is held against a picture of what
that frame should be, so a change to the yard, the beams or the wires is
seen rather than shipped.

## Why it belongs here

Spec 0001 says what the yard is and spec 0002 says what the wires do, and
every test of either is arithmetic: a position, a crossing, a catch. None of
them can see the yard.

The engine's spec 0076 made the rest of it one call, so what a game writes is
the reference, the size and what to draw.

**And the staging was already here.** `--screenshot` puts the camera over the
yard looking down, because from inside it you see a crate and a beam, and
what this game is, is the whole yard at once with the wires strung across it
and the light falling in two places. That is the frame
`refresh-screenshots` photographs, so this holds the picture that is already
on show.

## Behavior

**`SweepGame::new` takes `staged` rather than reading the arguments where the
camera is chosen.** A picture test asks for the photographed view without
`--screenshot` in its own command line. `main` passes what the arguments say,
which is the only place that reads them.

**One `update`, at no time at all.** The beams are placed and the lights
worked out, and a `dt` of nought means nothing sweeps between that and the
shutter.

**The picture is the 3D.** The engine's harness leaves the overlay empty, so
the distance left and the line of help are not in shot and a reference does
not change when either does.

**It is the engine's size**, 320 by 240, said once in `picture::SIZE` and
read here.

## Acceptance criteria

- The yard from above matches its
  picture. — `sweep_game::tests::the_yard_from_above_looks_right`

### Verified by hand

- **The picture itself.** The green finish across the top, two red wires
  across the yard, the crates and the pools the beams throw: the same view
  the committed `media/screenshot.png` shows, cropped to four by three and
  without the window or the readout.
- **A control.** The staged camera dropped from 0.95 of the yard's depth to
  0.90 and the test failed on 10.6% of its pixels. Put back and compared.

## Out of scope

**The readout.** It is 2D and the engine's harness draws none of it.

**A picture from inside the yard.** That is the view the game is played
from, and it shows a crate and a beam, which is why the staged camera exists
at all.
