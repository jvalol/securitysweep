//! The yard, the crates in it, and the two sides. See
//! `specs/0001-crossing-the-yard.md`.
//!
//! One set of boxes. What stops you walking is what stops the light, so
//! shelter is shelter and nothing has to be kept in step.

use blitzkit::collision::Aabb;
use blitzkit::mesh::MeshData;
use glam::{vec3, Vec3};

/// How wide the yard is across, and how deep from near side to far.
pub const WIDE: f32 = 30.0;
pub const DEEP: f32 = 40.0;

/// How high a crate stands, and how tall the wall round the yard is.
pub const CRATE_TALL: f32 = 2.2;
pub const WALL_TALL: f32 = 4.0;
const WALL_THICK: f32 = 1.0;

/// How far in from each end you start and finish.
const END: f32 = 3.0;

/// How deep the line you are walking to is drawn, and how far off the floor.
const FINISH_DEEP: f32 = 1.0;
const FINISH_TALL: f32 = 0.06;

/// Where you begin, at the near side.
pub fn start() -> Vec3 {
    vec3(0.0, 0.0, DEEP * 0.5 - END)
}

/// Whether a position has reached the far side.
pub fn is_across(at: Vec3) -> bool {
    at.z <= -(DEEP * 0.5 - END)
}

/// The crates, placed rather than generated, as boxes on the ground.
///
/// Placed so that no beam owns a whole width of the yard on its own: every
/// beam's ground has something in it to stand behind.
pub fn crates() -> Vec<Aabb> {
    let placed: [(f32, f32, f32, f32); 8] = [
        (-9.0, 13.0, 4.0, 3.0),
        (5.0, 11.0, 5.0, 3.0),
        (-2.0, 5.0, 3.0, 4.0),
        (-11.0, 1.0, 3.0, 5.0),
        (9.0, 0.0, 4.0, 4.0),
        (0.0, -6.0, 5.0, 3.0),
        (-8.0, -11.0, 4.0, 4.0),
        (8.0, -13.0, 3.0, 3.0),
    ];

    placed
        .iter()
        .copied()
        .map(|(x, z, wide, deep)| {
            Aabb::from_center_size(vec3(x, CRATE_TALL * 0.5, z), vec3(wide, CRATE_TALL, deep))
        })
        .collect()
}

/// The wall round the yard, so you cross it rather than walking off it.
pub fn walls() -> Vec<Aabb> {
    let half = vec3(WIDE * 0.5, 0.0, DEEP * 0.5);
    let up = Vec3::Y * WALL_TALL * 0.5;

    [
        (
            vec3(0.0, 0.0, -half.z),
            vec3(WIDE + WALL_THICK, WALL_TALL, WALL_THICK),
        ),
        (
            vec3(0.0, 0.0, half.z),
            vec3(WIDE + WALL_THICK, WALL_TALL, WALL_THICK),
        ),
        (
            vec3(-half.x, 0.0, 0.0),
            vec3(WALL_THICK, WALL_TALL, DEEP + WALL_THICK),
        ),
        (
            vec3(half.x, 0.0, 0.0),
            vec3(WALL_THICK, WALL_TALL, DEEP + WALL_THICK),
        ),
    ]
    .iter()
    .map(|(at, size)| Aabb::from_center_size(*at + up, *size))
    .collect()
}

/// Everything solid: the crates and the wall.
///
/// The same list stops you walking and stops a beam, which is the point.
pub fn solid() -> Vec<Aabb> {
    let mut all = crates();
    all.extend(walls());
    all
}

/// Where the line you are walking to lies, across the far end.
pub fn finish() -> Aabb {
    Aabb::from_center_size(
        vec3(0.0, FINISH_TALL * 0.5, -(DEEP * 0.5 - END)),
        vec3(WIDE, FINISH_TALL, FINISH_DEEP),
    )
}

/// Some boxes as one mesh.
fn boxes(all: &[Aabb]) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for bounds in all {
        let cube = MeshData::cube();
        let first = vertices.len() as u32;
        let (centre, size) = (bounds.center(), bounds.size());

        for vertex in &cube.vertices {
            let at = Vec3::from(vertex.position) * size + centre;
            vertices.push(blitzkit::mesh::Vertex::new(
                at.to_array(),
                vertex.normal,
                vertex.uv,
            ));
        }
        indices.extend(cube.indices.iter().map(|index| index + first));
    }

    MeshData::new(vertices, indices)
}

/// The ground, the crates and the wall as three meshes rather than one.
///
/// One mesh takes one colour, and one colour for the floor, the things standing
/// on it and the wall round it is a grey yard where nothing reads as anything.
pub fn ground_mesh() -> MeshData {
    boxes(&[Aabb::from_center_size(
        vec3(0.0, -0.5, 0.0),
        vec3(WIDE, 1.0, DEEP),
    )])
}

pub fn crates_mesh() -> MeshData {
    boxes(&crates())
}

pub fn wall_mesh() -> MeshData {
    boxes(&walls())
}

/// The line across the far end, so the way to win is something you can see from
/// where you start rather than something you have to be told.
pub fn finish_mesh() -> MeshData {
    boxes(&[finish()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_is_solid_both_ways() {
        // the boxes the beams cannot pass are the boxes you cannot walk
        // through, so what looks like shelter is shelter
        let solid = solid();

        for crate_ in crates() {
            assert!(
                solid.iter().any(|b| b.center() == crate_.center()),
                "a crate is not in the solid list"
            );
        }
        assert_eq!(solid.len(), crates().len() + walls().len());
    }

    #[test]
    fn you_start_at_the_near_side_and_finish_at_the_far() {
        assert!(!is_across(start()), "you start already across");
        assert!(is_across(vec3(0.0, 0.0, -(DEEP * 0.5 - END))));
        assert!(start().z > 0.0, "the near side is the positive one");
    }

    #[test]
    fn nothing_is_in_the_way_of_where_you_stand() {
        let you = Aabb::from_center_size(start() + Vec3::Y, Vec3::splat(1.0));

        for bounds in solid() {
            assert!(!bounds.intersects(&you), "you start inside something");
        }
    }

    #[test]
    fn there_is_a_way_across() {
        // a yard you cannot cross is the one bug a player cannot tell from
        // being bad at it, so this walks it: a grid of standing places, a clock
        // for the beams, and a search over both at once. Moving a cell a tick
        // is slower than the player walks, so anything this finds, they can do.
        use crate::{beams, seen, wires};
        use std::collections::VecDeque;

        const STEP: f32 = 2.0;
        const TICK: f32 = 0.5;
        const TICKS: usize = 150;

        let across = (WIDE / STEP) as usize;
        let down = (DEEP / STEP) as usize;
        let cover = solid();
        let wires = wires::all();

        let place = |x: usize, z: usize| {
            vec3(
                -WIDE * 0.5 + STEP * (x as f32 + 0.5),
                seen::CHEST,
                -DEEP * 0.5 + STEP * (z as f32 + 0.5),
            )
        };

        // where you can stand at all
        let standable: Vec<bool> = (0..across * down)
            .map(|cell| {
                let at = place(cell % across, cell / across);
                let you = Aabb::from_center_size(at, vec3(0.9, 1.8, 0.9));
                !cover.iter().any(|box_| box_.intersects(&you))
            })
            .collect();

        // and when, with the beams where they will be
        let mut clock = beams::all();
        let mut dark = vec![false; across * down * TICKS];
        for tick in 0..TICKS {
            let lights: Vec<_> = clock.iter().map(|beam| beam.light()).collect();
            for cell in 0..across * down {
                let at = place(cell % across, cell / across);
                dark[tick * across * down + cell] =
                    standable[cell] && !seen::seen(&lights, at, &cover);
            }
            for beam in clock.iter_mut() {
                beam.advance(TICK);
            }
        }

        // and the yard has to actually threaten, or a way across proves
        // nothing: a yard nothing ever lights is crossable by standing up.
        //
        // Beams and wires together, because the two share the pressure. Adding
        // wires and narrowing the beams to fit them must not quietly buy an
        // easy yard, and counting only one of them would let it.
        let standing_room: usize = standable.iter().filter(|room| **room).count();
        let on_a_wire = |cell: usize| {
            let mid = place(cell % across, cell / across);
            wires
                .iter()
                .any(|wire| wire.touching(vec3(mid.x, 0.0, mid.z)))
        };
        let threatened = (0..TICKS)
            .map(|tick| {
                (0..across * down)
                    .filter(|cell| {
                        standable[*cell] && (!dark[tick * across * down + cell] || on_a_wire(*cell))
                    })
                    .count()
            })
            .max()
            .unwrap_or(0);

        assert!(
            threatened * 5 > standing_room,
            "at its worst the yard threatens {} of {} standing places, which is \
             not a yard worth crossing",
            threatened,
            standing_room
        );

        let from = {
            let at = start();
            let x = ((at.x + WIDE * 0.5) / STEP) as usize;
            let z = ((at.z + DEEP * 0.5) / STEP) as usize;
            z.min(down - 1) * across + x.min(across - 1)
        };

        assert!(dark[from], "you are caught where you start");
        assert!(
            !wires.iter().any(|wire| wire.touching(start())),
            "you start on a wire"
        );

        // a cell a tick, or standing still
        let mut been = vec![false; across * down * TICKS];
        let mut edge = VecDeque::from([(from, 0usize)]);
        been[from] = true;
        let mut made_it = false;
        let mut furthest = f32::INFINITY;

        while let Some((cell, tick)) = edge.pop_front() {
            furthest = furthest.min(place(cell % across, cell / across).z);
            if place(cell % across, cell / across).z <= -(DEEP * 0.5 - END) {
                made_it = true;
                break;
            }
            if tick + 1 >= TICKS {
                continue;
            }

            let (x, z) = (cell % across, cell / across);
            let mut go = vec![(x, z)];
            if x > 0 {
                go.push((x - 1, z));
            }
            if x + 1 < across {
                go.push((x + 1, z));
            }
            if z > 0 {
                go.push((x, z - 1));
            }
            if z + 1 < down {
                go.push((x, z + 1));
            }

            for (nx, nz) in go {
                let next = nz * across + nx;
                let at = (tick + 1) * across * down + next;

                // spec 0002's wires cut the yard, so a step that crosses one is
                // not a step. Without this the search proves a route through
                // them that nobody can walk.
                let feet = |cell: usize| {
                    let mid = place(cell % across, cell / across);
                    vec3(mid.x, 0.0, mid.z)
                };
                if wires::tripped(&wires, feet(cell), feet(next)) {
                    continue;
                }

                if dark[at] && !been[at] {
                    been[at] = true;
                    edge.push_back((next, tick + 1));
                }
            }
        }

        assert!(
            made_it,
            "no way across the yard in {} seconds of the beams: got to z {:.0}, \
             and the line is at z {:.0}",
            TICKS as f32 * TICK,
            furthest,
            -(DEEP * 0.5 - END)
        );
    }

    #[test]
    fn the_wires_leave_a_way_past_each_of_them() {
        // a wire with no way round it is a wall, and a wall is this spec's job
        // rather than spec 0002's
        use crate::wires;

        const STEP: f32 = 1.0;
        let cover = solid();

        for wire in wires::all() {
            let past = (0..(WIDE / STEP) as usize)
                .map(|x| -WIDE * 0.5 + STEP * (x as f32 + 0.5))
                .filter(|x| {
                    let here = vec3(*x, 0.0, wire.bounds.center().z);
                    let you = Aabb::from_center_size(here + Vec3::Y, vec3(0.9, 1.8, 0.9));

                    !wire.touching(here) && !cover.iter().any(|box_| box_.intersects(&you))
                })
                .count();

            assert!(
                past > 0,
                "a wire at {:?} has no way past",
                wire.bounds.center()
            );
        }
    }

    #[test]
    fn the_finish_is_where_crossing_counts() {
        // the line you can see and the line the rule uses are the same line
        let line = finish();
        let just_past = vec3(0.0, 0.0, line.center().z - 0.01);
        let just_short = vec3(0.0, 0.0, line.center().z + 0.01);

        assert!(is_across(just_past), "past the line does not count");
        assert!(!is_across(just_short), "short of the line counts");
    }

    #[test]
    fn the_finish_runs_the_width_of_the_yard() {
        // a line you have to find is not a line you can see from the start
        assert!(finish().size().x >= WIDE - 1e-3);
    }

    #[test]
    fn the_yard_is_drawn_in_pieces() {
        // one mesh takes one colour, and one colour for all of it is a grey
        // yard where nothing reads as anything
        for mesh in [ground_mesh(), crates_mesh(), wall_mesh(), finish_mesh()] {
            assert!(!mesh.vertices.is_empty(), "an empty piece");
        }
        assert_eq!(crates_mesh().vertices.len(), 24 * crates().len());
    }

    #[test]
    fn the_crates_are_inside_the_yard() {
        for bounds in crates() {
            assert!(bounds.min.x > -WIDE * 0.5 && bounds.max.x < WIDE * 0.5);
            assert!(bounds.min.z > -DEEP * 0.5 && bounds.max.z < DEEP * 0.5);
        }
    }
}
