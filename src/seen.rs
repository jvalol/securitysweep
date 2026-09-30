//! Whether a beam has you. See `specs/0001-crossing-the-yard.md`.
//!
//! Three questions, and the engine answers all three: in the cone, from
//! `SpotLight::cone`; in range, from `SpotLight::falloff`; and nothing in the
//! way, from a ray against the cover, per blitzkit spec 0014.
//!
//! This is the whole reason this game exists. Every one of those also feeds the
//! shader, so what the rule decides is what the screen draws, and the rule can
//! be checked without a window.

use blitzkit::collision::{Aabb, Ray};
use blitzkit::lighting::SpotLight;
use glam::Vec3;

/// How much light counts as being seen.
///
/// Cone times falloff, both of which the shader multiplies together too. Low
/// enough that the soft edge of a beam is not a safe place to stand, high
/// enough that the very last of the falloff is.
pub const SEEN: f32 = 0.06;

/// How high up you are caught. Your feet are on the floor and the light sweeps
/// over the top of a crate, so what matters is your middle rather than either
/// end of you.
pub const CHEST: f32 = 1.0;

/// How much of this beam reaches you, ignoring what is in the way.
pub fn reaching(beam: &SpotLight, at: Vec3) -> f32 {
    let toward = at - beam.position;

    beam.cone(toward) * beam.falloff(toward.length())
}

/// Whether anything solid stands between the beam and you.
pub fn in_the_clear(beam: &SpotLight, at: Vec3, cover: &[Aabb]) -> bool {
    let toward = at - beam.position;
    let reach = toward.length();

    if reach < f32::EPSILON {
        return true;
    }

    !cover.iter().any(|box_| {
        Ray::new(beam.position, toward)
            .hit_aabb(box_)
            .is_some_and(|hit| hit.distance < reach - 1e-3)
    })
}

/// Whether this beam has you: in the cone, in range, and in the clear.
pub fn seen_by(beam: &SpotLight, at: Vec3, cover: &[Aabb]) -> bool {
    reaching(beam, at) > SEEN && in_the_clear(beam, at, cover)
}

/// Whether any of them has you.
pub fn seen(beams: &[SpotLight], at: Vec3, cover: &[Aabb]) -> bool {
    beams.iter().any(|beam| seen_by(beam, at, cover))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;

    fn overhead() -> SpotLight {
        SpotLight::new(
            vec3(0.0, 10.0, 0.0),
            Vec3::NEG_Y,
            Vec3::ONE,
            2.4,
            26.0,
            0.16,
            0.30,
        )
    }

    #[test]
    fn in_the_beam_is_seen() {
        assert!(seen_by(&overhead(), vec3(0.0, CHEST, 0.0), &[]));
    }

    #[test]
    fn outside_the_cone_is_not_seen() {
        // ten up and twelve sideways is well past a cone of a third of a radian
        assert!(!seen_by(&overhead(), vec3(12.0, CHEST, 0.0), &[]));
    }

    #[test]
    fn past_the_range_is_not_seen() {
        let beam = overhead();
        let far = vec3(0.0, 10.0 - beam.range - 1.0, 0.0);

        assert!(!seen_by(&beam, far, &[]));
    }

    #[test]
    fn cover_stops_the_beam() {
        let beam = overhead();
        let you = vec3(0.0, CHEST, 0.0);
        let lid = Aabb::from_center_size(vec3(0.0, 4.0, 0.0), vec3(4.0, 1.0, 4.0));

        assert!(seen_by(&beam, you, &[]), "not seen even in the open");
        assert!(!seen_by(&beam, you, &[lid]), "seen through a crate");
    }

    #[test]
    fn the_rule_uses_the_engines_cone() {
        // not its own arithmetic: the number the rule turns on is the one the
        // shader gets, so the screen and the test cannot drift apart
        let beam = overhead();
        let at = vec3(1.0, CHEST, 0.0);
        let toward = at - beam.position;

        assert_eq!(
            reaching(&beam, at),
            beam.cone(toward) * beam.falloff(toward.length())
        );
    }

    #[test]
    fn cover_behind_you_does_not_hide_you() {
        // a box past you is not between you and the beam
        let beam = overhead();
        let you = vec3(0.0, CHEST, 0.0);
        let behind = Aabb::from_center_size(vec3(0.0, -4.0, 0.0), vec3(4.0, 1.0, 4.0));

        assert!(seen_by(&beam, you, &[behind]));
    }

    #[test]
    fn any_beam_is_enough() {
        let one = overhead();
        let other = SpotLight::new(
            vec3(30.0, 10.0, 0.0),
            Vec3::NEG_Y,
            Vec3::ONE,
            2.4,
            26.0,
            0.16,
            0.30,
        );
        let you = vec3(0.0, CHEST, 0.0);

        assert!(!seen_by(&other, you, &[]), "the far one should miss");
        assert!(seen(&[other, one], you, &[]), "the near one should catch");
        assert!(!seen(&[other], you, &[]));
    }
}
