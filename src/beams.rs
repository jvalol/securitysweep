//! The four lights, sweeping and fixed. See
//! `specs/0001-crossing-the-yard.md`.

use blitzkit::lighting::{SpotLight, MAX_SPOT_LIGHTS};
use glam::{vec3, Vec3};

use crate::yard;

/// How many there are. Spec 0021 holds four spots and a shadow map for each,
/// and four is enough: a yard with two has one gap and a yard with six has
/// none.
pub const BEAMS: usize = MAX_SPOT_LIGHTS;

/// How high they hang, how far they reach, and how wide the cone is.
pub const HANGS: f32 = 9.0;
pub const RANGE: f32 = 26.0;
pub const INNER: f32 = 0.26;
pub const OUTER: f32 = 0.42;

pub const COLOR: Vec3 = vec3(1.0, 0.97, 0.88);
pub const INTENSITY: f32 = 2.4;

/// One light. A sweeping one is a gap that comes and goes; a fixed one is a
/// place you cannot be.
#[derive(Debug, Clone, Copy)]
pub struct Beam {
    pub at: Vec3,
    /// Which way it points across the yard, in radians about the upright.
    pub aim: f32,
    /// The two angles it swings between, or nothing at all if it holds still.
    pub between: Option<(f32, f32)>,
    /// Radians a second, and which way round it is going.
    pub rate: f32,
    going: f32,
}

impl Beam {
    pub fn sweeping(at: Vec3, from: f32, to: f32, rate: f32) -> Self {
        Self {
            at,
            aim: from,
            between: Some((from.min(to), from.max(to))),
            rate,
            going: 1.0,
        }
    }

    pub fn fixed(at: Vec3, aim: f32) -> Self {
        Self {
            at,
            aim,
            between: None,
            rate: 0.0,
            going: 0.0,
        }
    }

    /// Swings it on, turning round at either limit.
    ///
    /// It takes no argument but the time. A beam sweeps the same way whether
    /// you are there or not, which is what makes learning them the game.
    pub fn advance(&mut self, dt: f32) {
        let Some((low, high)) = self.between else {
            return;
        };

        self.aim += self.rate * self.going * dt;

        if self.aim >= high {
            self.aim = high;
            self.going = -1.0;
        } else if self.aim <= low {
            self.aim = low;
            self.going = 1.0;
        }
    }

    /// Which way the cone points: down the yard and tilted by its aim.
    pub fn facing(&self) -> Vec3 {
        let (sin, cos) = self.aim.sin_cos();

        vec3(sin, -1.0, cos).normalize()
    }

    pub fn light(&self) -> SpotLight {
        SpotLight::new(
            self.at,
            self.facing(),
            COLOR,
            INTENSITY,
            RANGE,
            INNER,
            OUTER,
        )
    }
}

/// The four, placed rather than generated.
pub fn all() -> Vec<Beam> {
    let high = yard::WALL_TALL + HANGS;

    let mut placed = vec![
        Beam::sweeping(vec3(-10.0, high, 12.0), -0.5, 0.7, 0.42),
        Beam::sweeping(vec3(9.0, high, 3.0), -0.7, 0.5, 0.31),
        Beam::sweeping(vec3(-7.0, high, -9.0), -0.4, 0.8, 0.55),
        Beam::fixed(vec3(6.0, high, -15.0), -0.2),
    ];

    // a fifth would not be an error, it would be a beam that quietly does not
    // light anything, so it is cut here rather than dropped there
    placed.truncate(BEAMS);
    placed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_beam_sweeps_and_turns_round() {
        let mut beam = Beam::sweeping(Vec3::ZERO, -0.5, 0.5, 1.0);
        let mut lowest = f32::INFINITY;
        let mut highest = f32::NEG_INFINITY;

        for _ in 0..600 {
            beam.advance(0.01);
            lowest = lowest.min(beam.aim);
            highest = highest.max(beam.aim);
        }

        assert!(highest > 0.4, "it never reached the far end: {}", highest);
        assert!(lowest < -0.4, "it never came back: {}", lowest);
        assert!(beam.aim >= -0.5 - 1e-5 && beam.aim <= 0.5 + 1e-5);
    }

    #[test]
    fn a_fixed_beam_holds_still() {
        let mut beam = Beam::fixed(Vec3::ZERO, 0.3);

        for _ in 0..100 {
            beam.advance(0.1);
        }

        assert_eq!(beam.aim, 0.3);
    }

    #[test]
    fn there_are_never_more_than_four() {
        // spec 0021 takes four, and the rest would be dropped in silence
        assert_eq!(BEAMS, MAX_SPOT_LIGHTS);
        assert!(all().len() <= BEAMS, "{} beams", all().len());
    }

    #[test]
    fn a_beam_does_not_watch_you() {
        // advancing takes the time and nothing else, so two of them run the
        // same however differently anyone moves
        let mut one = Beam::sweeping(Vec3::ZERO, -0.5, 0.5, 0.7);
        let mut other = one;

        for _ in 0..200 {
            one.advance(0.02);
            other.advance(0.02);
        }

        assert_eq!(one.aim, other.aim);
    }

    #[test]
    fn a_beam_points_down_into_the_yard() {
        for beam in all() {
            assert!(beam.facing().y < 0.0, "a beam points up");
            assert!(beam.at.y > yard::WALL_TALL, "a beam hangs below the wall");
        }
    }
}
