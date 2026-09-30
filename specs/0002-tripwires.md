# 0002 Tripwires

**Status:** implemented
**Date:** 2026-09-30

## Goal

Spec 0001 gives the yard one kind of threat and one kind of decision: a beam is
an area that moves, and the question is when to go. Something that asks a
different question.

## Behavior

**A wire is a line that does not move and gives you nothing.** A beam is an area
that moves and gives you a meter's worth of grace. One is timing and the other
is routing, and having both means they compose: find the gap, then wait for the
sweep.

**They are not lights.** Spec 0021 holds four spots and every one of them casts,
so a wire made of light would cost a beam. It would also be wrong: a tripwire
does not light a room. They are drawn with an unclamped colour the way the
finish line is, and they are tested against your body rather than looked up in
the lighting.

**Crossing one is a step, not a place.** Tested where you stand, a wire is a
wire you can run through: at four and a half units a second a frame carries you
most of a stride, and a thin line falls between two of them. The step from where
you were to where you are is swept against it instead, which is blitzkit spec
0014's swept sphere doing what it already does for walls.

**Catching you is your body, not your middle.** A wire you can stand on top of
without setting it off is a wire nobody believes in.

**None of them runs the whole width.** A wire with no way past is a wall, and
walls are spec 0001's job. Each leaves a gap, and the gap is the routing
decision.

**The gaps have to be somewhere you can get to.** A gap behind a crate, or in a
lane a beam never leaves, is not a gap. Spec 0001's crossing search covers this
now: a step that crosses a wire is not a step, so the search proves a route
through the beams *and* the wires or it proves nothing.

**Beams and wires share the pressure.** Adding wires and leaving the beams as
they were made the yard uncrossable, so the cones came in from 0.42 to 0.34. The
test that holds the yard dangerous counts both, so narrowing the beams to fit
the wires cannot quietly buy an easy yard.

## Acceptance criteria

- Standing on a wire trips it. — `wires::tests::standing_on_a_wire_trips_it`
- Standing clear of one does not. — `wires::tests::standing_clear_of_one_does_not`
- Walking through one trips it. — `wires::tests::walking_through_one_trips_it`
- And a step clean over one still does. — `wires::tests::a_step_clean_over_one_still_trips_it`
- Walking beside one does not. — `wires::tests::walking_beside_one_does_not_trip_it`
- Walking up to one and stopping does not. — `wires::tests::walking_up_to_one_and_stopping_does_not`
- Standing still is tested where you stand. — `wires::tests::standing_still_is_tested_where_you_stand`
- None of them runs the whole width. — `wires::tests::none_of_them_runs_the_whole_width`
- They hang where they can catch you. — `wires::tests::they_hang_where_they_can_catch_you`
- Any one of them is enough. — `wires::tests::any_wire_is_enough`
- Each leaves somewhere you can walk past it. — `yard::tests::the_wires_leave_a_way_past_each_of_them`
- There is still a way across, with the wires as well as the beams. — `yard::tests::there_is_a_way_across`
- A wire sends you back with no meter to fill. — `sweep_game::tests::a_wire_sends_you_back_with_no_meter_to_fill`

### Verified by hand

- A wire reads as a line to duck round rather than as decoration.
- Walking into one and being sent back feels like your mistake.
- The gap in a wire is findable without hunting for it.

## Out of scope

Wires that move, blink, or turn off. Wires you can crawl under or climb over.
Anything that reacts to one being tripped beyond sending you back.
