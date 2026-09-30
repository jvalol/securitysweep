//! Standing in the yard and walking across it. See
//! `specs/0001-crossing-the-yard.md`.

use blitzkit::collision::{move_and_slide, Aabb, Sphere};
use glam::{vec3, Vec3};

/// How wide you are, how high your eye is, and how fast you walk.
///
/// Slower than the search in `yard` assumes, so anything it proves you can
/// cross, you can actually cross.
pub const RADIUS: f32 = 0.45;
pub const EYE: f32 = 1.6;
pub const SPEED: f32 = 4.4;

pub const LOOK: f32 = 0.0022;
pub const TURN: f32 = 2.2;

/// How far up and down you can look, short of straight up.
const PITCH_LIMIT: f32 = 1.45;

#[derive(Debug, Clone, Copy)]
pub struct Walker {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Walker {
    pub fn at(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn facing(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();

        vec3(sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch)
    }

    pub fn forward(&self) -> Vec3 {
        let (sin, cos) = self.yaw.sin_cos();

        vec3(sin, 0.0, -cos)
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y)
    }

    pub fn eye(&self) -> Vec3 {
        self.position + Vec3::Y * EYE
    }

    /// Where a beam catches you: your middle, not your feet.
    pub fn chest(&self) -> Vec3 {
        self.position + Vec3::Y * crate::seen::CHEST
    }

    pub fn turn(&mut self, amount: f32) {
        self.yaw += amount;
    }

    pub fn look(&mut self, delta: glam::Vec2) {
        self.yaw += delta.x * LOOK;
        self.pitch = (self.pitch - delta.y * LOOK).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    pub fn walk(&mut self, wish: Vec3, dt: f32, solid: &[Aabb]) {
        if wish.length_squared() < 1e-6 {
            return;
        }

        let body = Sphere::new(self.position + Vec3::Y * RADIUS, RADIUS);
        let moved = move_and_slide(body, wish.normalize() * SPEED, dt, solid);

        self.position = moved - Vec3::Y * RADIUS;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crate_at(x: f32) -> Aabb {
        Aabb::from_center_size(vec3(x, 1.0, 0.0), vec3(3.0, 2.0, 3.0))
    }

    #[test]
    fn a_crate_stops_you() {
        let mut you = Walker::at(Vec3::ZERO);

        for _ in 0..60 {
            you.walk(Vec3::X, 0.05, &[crate_at(4.0)]);
        }

        assert!(you.position.x < 2.5, "walked to {}", you.position.x);
    }

    #[test]
    fn you_slide_along_a_crate() {
        let mut you = Walker::at(Vec3::ZERO);
        let before = you.position.z;

        for _ in 0..40 {
            you.walk(vec3(1.0, 0.0, 1.0), 0.05, &[crate_at(4.0)]);
        }

        assert!(you.position.z > before + 1.0);
    }

    #[test]
    fn walking_is_flat_however_you_look() {
        let mut you = Walker::at(Vec3::ZERO);
        you.pitch = -1.0;

        you.walk(you.forward(), 0.1, &[]);

        assert!(you.position.y.abs() < 1e-5);
    }

    #[test]
    fn a_beam_catches_your_middle_not_your_feet() {
        let you = Walker::at(Vec3::ZERO);

        assert!(you.chest().y > 0.0, "it would catch the floor");
        assert!(you.chest().y < EYE, "it would catch over a crate");
    }

    #[test]
    fn you_walk_slower_than_the_search_assumes() {
        // yard's crossing search moves two units every half second, so if you
        // were slower than that it could prove a crossing you cannot make
        let mut you = Walker::at(Vec3::ZERO);
        you.walk(Vec3::X, 0.5, &[]);

        assert!(
            you.position.x >= 2.0,
            "you covered {} in the half second the search gives you two",
            you.position.x
        );
    }
}
