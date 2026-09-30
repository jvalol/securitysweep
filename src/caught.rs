//! The meter, and being sent back. See `specs/0001-crossing-the-yard.md`.
//!
//! A meter rather than a verdict: a beam clipping you for half a second is
//! survivable, so the question is how much light you will take rather than
//! only when the gap comes.

/// How long in the full light it takes to fill from empty, in seconds.
pub const FILLS_IN: f32 = 1.6;

/// And how long in the dark it takes to empty again.
///
/// Longer than it fills. A long walk through the edge of a beam has to be worse
/// than a dash through the middle of one, or the edge is simply a slower safe
/// route and there is no decision in it.
pub const EMPTIES_IN: f32 = 4.0;

/// How much of it is filled, from nothing to caught.
#[derive(Debug, Clone, Copy, Default)]
pub struct Meter {
    filled: f32,
}

impl Meter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filled(&self) -> f32 {
        self.filled
    }

    pub fn empty(&mut self) {
        self.filled = 0.0;
    }

    /// Runs it on, and says whether that caught you.
    ///
    /// Caught empties it, because the next thing that happens is standing at
    /// the near side again and a meter still full would catch you twice.
    pub fn update(&mut self, lit: bool, dt: f32) -> bool {
        let rate = if lit {
            1.0 / FILLS_IN
        } else {
            -1.0 / EMPTIES_IN
        };

        self.filled = (self.filled + rate * dt).clamp(0.0, 1.0);

        if self.filled >= 1.0 {
            self.filled = 0.0;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_fills_it_and_dark_empties_it() {
        let mut meter = Meter::new();

        meter.update(true, 0.5);
        let lit = meter.filled();
        assert!(lit > 0.0, "the light did nothing");

        meter.update(false, 0.5);
        assert!(meter.filled() < lit, "the dark did nothing");
    }

    #[test]
    fn it_empties_slower_than_it_fills() {
        let mut one = Meter::new();
        one.update(true, 0.4);
        let gained = one.filled();

        let mut other = Meter::new();
        other.update(true, 1.0);
        other.update(false, 0.4);
        let lost = 1.0 / FILLS_IN - other.filled();

        assert!(
            lost < gained,
            "it lost {} in the dark and gained {} in the light",
            lost,
            gained
        );
    }

    #[test]
    fn full_sends_you_back() {
        let mut meter = Meter::new();

        let caught = (0..100).any(|_| meter.update(true, 0.1));

        assert!(caught, "standing in the light forever never caught you");
    }

    #[test]
    fn being_caught_empties_it() {
        // or the next thing that happens is being caught again at once
        let mut meter = Meter::new();

        while !meter.update(true, 0.1) {}

        assert_eq!(meter.filled(), 0.0);
    }

    #[test]
    fn it_stays_between_empty_and_full() {
        let mut meter = Meter::new();

        for step in 0..500 {
            meter.update(step % 7 < 3, 0.05);
            assert!(
                (0.0..=1.0).contains(&meter.filled()),
                "it reached {}",
                meter.filled()
            );
        }
    }

    #[test]
    fn the_dark_cannot_take_it_below_empty() {
        let mut meter = Meter::new();

        for _ in 0..100 {
            meter.update(false, 1.0);
        }

        assert_eq!(meter.filled(), 0.0);
    }
}
