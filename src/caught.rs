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

/// How long the screen stays red after you are caught.
///
/// Short. A tint that lingers reads as a state you are in; a flash that is gone
/// before you have finished flinching reads as a thing that happened.
pub const FLASH_FOR: f32 = 0.3;

/// How red it goes at its reddest. Not opaque, because being unable to see the
/// yard is a second punishment for one mistake.
pub const FLASH_MOST: f32 = 0.5;

/// The red over the screen, fading.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flash {
    left: f32,
}

impl Flash {
    pub fn new() -> Self {
        Self::default()
    }

    /// Being caught, which starts it over however much was left.
    pub fn start(&mut self) {
        self.left = FLASH_FOR;
    }

    pub fn fade(&mut self, dt: f32) {
        self.left = (self.left - dt).max(0.0);
    }

    /// How opaque the red is now, from nothing to `FLASH_MOST`.
    pub fn alpha(&self) -> f32 {
        FLASH_MOST * (self.left / FLASH_FOR).clamp(0.0, 1.0)
    }
}

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
    fn being_caught_reddens_the_screen() {
        let mut flash = Flash::new();
        assert_eq!(flash.alpha(), 0.0, "red before anything happened");

        flash.start();
        assert!(flash.alpha() > 0.0);
    }

    #[test]
    fn the_red_fades_and_stops() {
        let mut flash = Flash::new();
        flash.start();

        // a step past the whole of it, because ten tenths of a float is not
        // reliably the whole of it
        let mut before = flash.alpha();
        for _ in 0..11 {
            flash.fade(FLASH_FOR / 10.0);
            assert!(flash.alpha() <= before, "it got redder as it faded");
            before = flash.alpha();
        }

        assert_eq!(flash.alpha(), 0.0, "it never finished");
    }

    #[test]
    fn it_never_goes_past_clear_or_past_red() {
        let mut flash = Flash::new();
        flash.start();

        for _ in 0..200 {
            flash.fade(0.1);
            assert!((0.0..=FLASH_MOST).contains(&flash.alpha()));
        }
    }

    #[test]
    fn being_caught_again_starts_it_over() {
        let mut flash = Flash::new();
        flash.start();
        flash.fade(FLASH_FOR * 0.8);
        let dim = flash.alpha();

        flash.start();

        assert!(
            flash.alpha() > dim,
            "the second catch was dimmer than the first"
        );
    }

    #[test]
    fn you_can_still_see_the_yard_through_it() {
        // being unable to see is a second punishment for one mistake
        let mut flash = Flash::new();
        flash.start();

        assert!(flash.alpha() < 1.0, "it is opaque at {}", flash.alpha());
    }

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
