# 0001 Crossing the yard

**Status:** implemented
**Date:** 2026-09-29

## Goal

An open yard with four lights sweeping it, and the far side. Lantern made light
the thing you ration; this makes it the thing you avoid.

## Behavior

**The yard is open, and you can see the whole problem from where you start.**
Not a maze. Nothing to explore and nothing to find: the beams, the cover and the
far side are all in front of you from the first second, so every move is a
decision rather than a discovery.

**Four beams, because four is what the engine takes.** Spec 0021 holds four
spots and a shadow map for each. Some sweep between two angles and turn round,
some hold still and simply own a piece of ground. A sweeping one is a gap that
comes and goes; a fixed one is a place you cannot be.

**Being seen is three questions, and the engine already answers all three.**

- In the cone, from `SpotLight::cone`.
- In range, from `SpotLight::falloff`.
- Nothing in the way, from a ray against the cover, per spec 0014.

This is the whole reason this game exists. Every one of those is a function the
shader uses too, so what the rule decides is what the screen draws, and the rule
can be checked without a window. A game whose losing condition is a lighting
question is a game whose losing condition has unit tests.

**A meter, not a verdict.** Light fills it and the dark empties it, and full
sends you back to the near side. A beam clipping you for half a second is
survivable, so the question is how much light you will take rather than only
when the gap comes. It empties slower than it fills, so a long way through the
edge of a beam is worse than a dash through the middle of one.

**Cover is solid both ways.** The crates you hide behind are the boxes you
cannot walk through, and the geometry the beams cannot pass. One set of boxes,
so what looks like shelter is shelter and nothing has to be kept in step.

**There is always a way across.** Beams and cover are placed, not generated, but
a test walks the yard and proves a route exists with the beams where they are.
A yard you cannot cross is the one bug a player cannot tell from being bad at
it.

**The far side is a line you can see.** It glows, because the yard is dark and
a finish you have to be told about is not one you can aim at. The line drawn and
the line the rule counts are the same line.

**The yard is drawn in pieces.** One mesh takes one colour, and the floor, the
crates standing on it and the wall round it in a single grey is a yard where
nothing reads as anything.

**Being caught says so.** The screen reddens and fades over a moment. Not
opaque, and under the meter rather than over it: being unable to see the yard is
a second punishment for one mistake, and so is losing the thing you were reading.

**Nothing chases you.** The lights are the only opposition, and they do not
react. They sweep the same way whether you are there or not, so learning them is
the game.

## Acceptance criteria

- In the cone, in range and in the clear is seen. — `seen::tests::in_the_beam_is_seen`
- Outside the cone is not. — `seen::tests::outside_the_cone_is_not_seen`
- Past the range is not. — `seen::tests::past_the_range_is_not_seen`
- Cover between you and a beam is not. — `seen::tests::cover_stops_the_beam`
- The rule asks the engine's own cone rather than its own arithmetic. — `seen::tests::the_rule_uses_the_engines_cone`
- A sweeping beam goes between its limits and turns round. — `beams::tests::a_beam_sweeps_and_turns_round`
- A fixed beam holds still. — `beams::tests::a_fixed_beam_holds_still`
- There are never more beams than spec 0021 takes. — `beams::tests::there_are_never_more_than_four`
- A beam sweeps the same whether you are there or not. — `beams::tests::a_beam_does_not_watch_you`
- Light fills the meter and the dark empties it. — `caught::tests::light_fills_it_and_dark_empties_it`
- It empties slower than it fills. — `caught::tests::it_empties_slower_than_it_fills`
- Full sends you back to the near side. — `caught::tests::full_sends_you_back`
- Being caught reddens the screen. — `caught::tests::being_caught_reddens_the_screen`
- The red fades, and stops. — `caught::tests::the_red_fades_and_stops`
- Being caught again starts it over. — `caught::tests::being_caught_again_starts_it_over`
- You can still see the yard through it. — `caught::tests::you_can_still_see_the_yard_through_it`
- It never goes past clear or past red. — `caught::tests::it_never_goes_past_clear_or_past_red`
- It never goes below empty or above full. — `caught::tests::it_stays_between_empty_and_full`
- Cover stops you walking as well as it stops the light. — `yard::tests::cover_is_solid_both_ways`
- A crate stops you, and you slide along it. — `walker::tests::a_crate_stops_you`
- You walk at least as fast as the crossing search assumes. — `walker::tests::you_walk_slower_than_the_search_assumes`
- Being caught keeps you facing the same way. — `sweep_game::tests::being_caught_keeps_you_facing_the_same_way`
- Being caught empties the meter. — `caught::tests::being_caught_empties_it`
- There is a way across, with the beams where they are. — `yard::tests::there_is_a_way_across`
- The line you see and the line that counts are the same line. — `yard::tests::the_finish_is_where_crossing_counts`
- It runs the width of the yard. — `yard::tests::the_finish_runs_the_width_of_the_yard`
- The yard is drawn in pieces, so it can be more than one colour. — `yard::tests::the_yard_is_drawn_in_pieces`

### Verified by hand

- The edge of a beam is somewhere you can stand for a moment and know it.
- A crate's shadow is where the crate is, so aiming for shelter works by eye.
- Watching from the start, the beams read as a pattern rather than as noise.
- Being sent back is annoying and not unfair, which is the difference between a
  game and a slot machine.

## Out of scope

Anything that reacts to you: guards, alarms, a beam that follows. A second yard.
Anything to collect. Crouching, sprinting, or any verb but walking.
