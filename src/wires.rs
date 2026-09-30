//! Tripwires: a line you must not cross. See
//! `specs/0002-tripwires.md`.
//!
//! Not lights. A beam is an area that moves and gives you a meter's worth of
//! grace; a wire is a line that does not move and gives you none. One is
//! timing and the other is routing, and having both means they compose: get
//! through the gap, then wait for the sweep.
//!
//! They are drawn rather than lit, with an unclamped colour the way the finish
//! line is, because a tripwire does not light a room.

use blitzkit::collision::{sweep_sphere, Aabb, Sphere};
use blitzkit::mesh::MeshData;
use glam::{vec3, Vec3};

use crate::yard;

/// How thick a wire is drawn, and how high off the ground it sits.
pub const THICK: f32 = 0.07;
pub const AT_HEIGHT: f32 = 0.9;

/// How near you have to be for it to catch you. Your body, not a point: a wire
/// you can stand on top of without setting off is a wire nobody believes in.
pub const CATCHES: f32 = 0.45;

/// One wire, as the box it occupies.
///
/// A box rather than a segment because everything else here is a box, and the
/// engine's swept sphere already knows how to meet one.
#[derive(Debug, Clone, Copy)]
pub struct Wire {
    pub bounds: Aabb,
}

impl Wire {
    /// A wire running across the yard at this depth, between these two sides.
    pub fn across(z: f32, from_x: f32, to_x: f32) -> Self {
        let (low, high) = (from_x.min(to_x), from_x.max(to_x));

        Self {
            bounds: Aabb::from_center_size(
                vec3((low + high) * 0.5, AT_HEIGHT, z),
                vec3(high - low, THICK, THICK),
            ),
        }
    }

    /// Whether standing here is touching it.
    pub fn touching(&self, at: Vec3) -> bool {
        let you = Aabb::from_center_size(at + Vec3::Y * AT_HEIGHT, Vec3::splat(CATCHES * 2.0));

        you.intersects(&self.bounds)
    }

    /// And whether walking from here to there crosses it.
    ///
    /// Standing still and testing every frame misses a wire you step clean over
    /// between two frames, which at four and a half units a second is a wire
    /// you can run through.
    pub fn crossed(&self, from: Vec3, to: Vec3) -> bool {
        let body = Sphere::new(from + Vec3::Y * AT_HEIGHT, CATCHES);
        let step = to - from;

        if step.length_squared() < 1e-9 {
            return self.touching(from);
        }

        sweep_sphere(&body, step, &self.bounds).is_some_and(|hit| hit.distance <= step.length())
    }
}

/// The wires, placed rather than generated.
///
/// They cut the yard into bands, so the route is a gap to find before the sweep
/// is a gap to time. None of them runs the whole width: a wire with no way past
/// is a wall, and a wall is spec 0001's job.
pub fn all() -> Vec<Wire> {
    let half = yard::WIDE * 0.5;

    vec![
        Wire::across(8.0, -half, 5.0),
        Wire::across(-3.0, -half, 5.0),
    ]
}

/// Whether any of them catches that step.
pub fn tripped(wires: &[Wire], from: Vec3, to: Vec3) -> bool {
    wires.iter().any(|wire| wire.crossed(from, to))
}

/// The wires as one mesh.
pub fn mesh() -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for wire in all() {
        let cube = MeshData::cube();
        let first = vertices.len() as u32;
        let (centre, size) = (wire.bounds.center(), wire.bounds.size());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_on_a_wire_trips_it() {
        let wire = Wire::across(0.0, -5.0, 5.0);

        assert!(wire.touching(Vec3::ZERO));
    }

    #[test]
    fn standing_clear_of_one_does_not() {
        let wire = Wire::across(0.0, -5.0, 5.0);

        assert!(!wire.touching(vec3(0.0, 0.0, 4.0)));
        assert!(!wire.touching(vec3(9.0, 0.0, 0.0)), "past its end");
    }

    #[test]
    fn walking_through_one_trips_it() {
        let wire = Wire::across(0.0, -5.0, 5.0);

        assert!(wire.crossed(vec3(0.0, 0.0, 2.0), vec3(0.0, 0.0, -2.0)));
    }

    #[test]
    fn a_step_clean_over_one_still_trips_it() {
        // a wire tested only where you stand is a wire you can run through
        // between two frames
        let wire = Wire::across(0.0, -5.0, 5.0);
        let from = vec3(0.0, 0.0, 3.0);
        let to = vec3(0.0, 0.0, -3.0);

        assert!(!wire.touching(from), "it starts on the wire");
        assert!(!wire.touching(to), "it ends on the wire");
        assert!(wire.crossed(from, to), "it stepped clean over");
    }

    #[test]
    fn walking_beside_one_does_not_trip_it() {
        let wire = Wire::across(0.0, -5.0, 5.0);

        assert!(!wire.crossed(vec3(9.0, 0.0, 2.0), vec3(9.0, 0.0, -2.0)));
    }

    #[test]
    fn walking_up_to_one_and_stopping_does_not() {
        let wire = Wire::across(0.0, -5.0, 5.0);
        let short = vec3(0.0, 0.0, CATCHES + THICK + 0.2);

        assert!(!wire.crossed(vec3(0.0, 0.0, 4.0), short));
    }

    #[test]
    fn standing_still_is_tested_where_you_stand() {
        let wire = Wire::across(0.0, -5.0, 5.0);

        assert!(wire.crossed(Vec3::ZERO, Vec3::ZERO));
        assert!(!wire.crossed(vec3(0.0, 0.0, 5.0), vec3(0.0, 0.0, 5.0)));
    }

    #[test]
    fn none_of_them_runs_the_whole_width() {
        // a wire with no way past is a wall, and walls are spec 0001's job
        let half = yard::WIDE * 0.5;

        for wire in all() {
            let size = wire.bounds.size();
            assert!(
                size.x < yard::WIDE - 1e-3 || size.z < yard::DEEP - 1e-3,
                "a wire spans the whole yard"
            );
            assert!(wire.bounds.min.x >= -half - 1e-3);
            assert!(wire.bounds.max.x <= half + 1e-3);
        }
    }

    #[test]
    fn they_hang_where_they_can_catch_you() {
        for wire in all() {
            assert!(wire.bounds.center().y > 0.0, "a wire is underground");
            assert!(
                wire.bounds.center().y < crate::walker::EYE,
                "a wire is over your head"
            );
        }
    }

    #[test]
    fn any_wire_is_enough() {
        // a place to stand is a place on the ground: `touching` raises it to
        // the wire's height itself
        let wires = all();
        let under = wires[0].bounds.center();
        let one = vec3(under.x, 0.0, under.z);

        assert!(tripped(&wires, one, one));
        assert!(!tripped(&wires, vec3(0.0, 0.0, 19.0), vec3(0.0, 0.0, 19.0)));
    }
}
