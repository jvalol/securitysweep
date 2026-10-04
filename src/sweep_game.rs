//! The yard, crossed. See `specs/0001-crossing-the-yard.md`.

use blitzkit::camera::Camera;
use blitzkit::collision::Aabb;
use blitzkit::geometry::quad::Quad;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::mesh::Transform;
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::Game;
use glam::{vec2, vec4, Vec2, Vec3};

use crate::beams::{self, Beam};
use crate::caught::{Flash, Meter};
use crate::seen;
use crate::walker::Walker;
use crate::wires::{self, Wire};
use crate::yard;

/// Four colours rather than one. A yard where the floor, the things standing on
/// it and the wall round it are the same grey is a yard where nothing reads as
/// anything.
const GROUND: glam::Vec4 = vec4(0.36, 0.34, 0.31, 1.0);
const CRATES: glam::Vec4 = vec4(0.52, 0.40, 0.26, 1.0);
const WALL: glam::Vec4 = vec4(0.20, 0.20, 0.23, 1.0);

/// The line you are walking to. Colour is an unclamped multiplier on the
/// ambient term, so a large one glows without a light on it, which is how it
/// stays visible from the near side of a dark yard.
const FINISH: glam::Vec4 = vec4(2.4, 5.6, 3.0, 1.0);

/// The wires, glowing the same way and in the one colour that says stop.
const WIRE: glam::Vec4 = vec4(7.0, 1.2, 1.4, 1.0);

/// The red over everything when you are caught.
///
/// Near full brightness, because alpha blending over a dark yard gives you the
/// colour times the alpha and nothing else. A dark red at half alpha came out
/// at a tenth of full and read as a block in the corner rather than a flash.
const FLASH: Vec3 = glam::vec3(1.0, 0.13, 0.12);

/// How much light there is with nothing lighting it. Low: the beams are the
/// scene, and a yard you can read in the dark has no beams worth avoiding.
const AMBIENT: f32 = 0.10;

/// The meter, drawn across the bottom.
const METER_WIDE: f32 = 320.0;
const METER_TALL: f32 = 10.0;
const METER_MARGIN: f32 = 24.0;
const METER_EMPTY: glam::Vec4 = vec4(0.16, 0.17, 0.20, 0.8);
const METER_FULL: glam::Vec4 = vec4(0.95, 0.35, 0.25, 0.9);

pub struct SweepGame {
    you: Walker,
    beams: Vec<Beam>,
    solid: Vec<Aabb>,
    wires: Vec<Wire>,
    meter: Meter,
    flash: Flash,
    across: bool,
    ground: Option<MeshId>,
    crates: Option<MeshId>,
    wall: Option<MeshId>,
    finish: Option<MeshId>,
    wire_mesh: Option<MeshId>,
    /// forward, back, left, right, turn left, turn right
    held: [bool; 6],
    locked: bool,
    wants_lock: bool,
    quitting: bool,
    width: f32,
    height: f32,
    readout: RenderText,
    controls: RenderText,
}

impl SweepGame {
    pub fn new() -> Self {
        Self {
            you: Walker::at(yard::start()),
            beams: beams::all(),
            solid: yard::solid(),
            wires: wires::all(),
            meter: Meter::new(),
            flash: Flash::new(),
            across: false,
            ground: None,
            crates: None,
            wall: None,
            finish: None,
            wire_mesh: None,
            held: [false; 6],
            locked: false,
            wants_lock: false,
            quitting: false,
            width: 0.0,
            height: 0.0,
            readout: RenderText {
                position: vec2(20.0, 20.0),
                color: vec4(1.0, 1.0, 1.0, 0.9),
                size: 20.0,
                ..Default::default()
            },
            controls: RenderText {
                position: vec2(20.0, 52.0),
                color: vec4(1.0, 1.0, 1.0, 0.55),
                size: 20.0,
                text: String::from(
                    "Press the left and right arrow keys to rotate, wasd keys or the mouse to move",
                ),
                ..Default::default()
            },
        }
    }

    /// The lights as the engine wants them.
    fn lights(&self) -> Vec<blitzkit::lighting::SpotLight> {
        self.beams.iter().map(|beam| beam.light()).collect()
    }

    /// Whether any beam has you, right now.
    fn lit(&self) -> bool {
        seen::seen(&self.lights(), self.you.chest(), &self.solid)
    }

    /// Back to the near side, and the meter with it.
    fn send_back(&mut self) {
        let yaw = self.you.yaw;
        self.you = Walker::at(yard::start());
        self.you.yaw = yaw;
        self.meter.empty();
        self.flash.start();
    }

    fn wish(&self) -> Vec3 {
        let [forward, back, left, right, _, _] = self.held;
        let mut wish = Vec3::ZERO;

        if forward {
            wish += self.you.forward();
        }
        if back {
            wish -= self.you.forward();
        }
        if right {
            wish += self.you.right();
        }
        if left {
            wish -= self.you.right();
        }

        wish
    }
}

impl Default for SweepGame {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for SweepGame {
    fn load(&mut self, renderer: &mut Renderer) {
        self.ground = Some(renderer.add_mesh(&yard::ground_mesh()));
        self.crates = Some(renderer.add_mesh(&yard::crates_mesh()));
        self.wall = Some(renderer.add_mesh(&yard::wall_mesh()));
        self.finish = Some(renderer.add_mesh(&yard::finish_mesh()));
        self.wire_mesh = Some(renderer.add_mesh(&wires::mesh()));
        renderer.set_scene_bounds(Aabb::from_center_size(
            Vec3::ZERO,
            glam::vec3(yard::WIDE, 24.0, yard::DEEP),
        ));
    }

    fn before_frame(&mut self, renderer: &mut Renderer) {
        if self.wants_lock != self.locked {
            self.locked = renderer.set_cursor_locked(self.wants_lock) && self.wants_lock;
        }
    }

    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        window_size: (f32, f32),
    ) {
        self.resized(window_size);
    }

    fn resized(&mut self, window_size: (f32, f32)) {
        self.width = window_size.0;
        self.height = window_size.1;
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        geometry.reset();
        text_renderer.reset();

        let [_, _, _, _, turn_left, turn_right] = self.held;
        if turn_left {
            self.you.turn(-crate::walker::TURN * dt);
        }
        if turn_right {
            self.you.turn(crate::walker::TURN * dt);
        }

        // where the step began, because a wire is crossed between two frames
        // rather than stood on at either end of one
        let was = self.you.position;
        self.you.walk(self.wish(), dt, &self.solid);

        // the beams take the time and nothing else: they sweep the same way
        // whether you are there or not
        for beam in self.beams.iter_mut() {
            beam.advance(dt);
        }

        if !self.across {
            // a wire is not a meter. A beam gives you a moment; a wire gives
            // you none, which is what makes it a different question. The meter
            // still runs on a frame a wire catches you, or standing on one in
            // a beam would leave it half full when you reappear.
            let tripped = wires::tripped(&self.wires, was, self.you.position);
            let filled = self.meter.update(self.lit(), dt);

            if tripped || filled {
                self.send_back();
            }
            self.across = yard::is_across(self.you.position);
        }

        self.flash.fade(dt);

        self.readout.text = if self.across {
            String::from("across")
        } else {
            format!("{:.0}m to go", self.you.position.z + yard::DEEP * 0.5)
        };

        let wide = (
            self.width - 40.0,
            blitzkit::renderer::render_text::UNBOUNDED_F32,
        );
        self.controls.bounds = wide.into();
        text_renderer.render_texts.push(self.readout.clone());
        text_renderer.render_texts.push(self.controls.clone());

        // the red goes over the yard but under the meter: it is there to say
        // what happened, not to take away the thing you were reading
        // a quad's position is its middle, not its corner
        if self.flash.alpha() > 0.0 {
            let whole = vec2(self.width, self.height);

            geometry.push_quad(&Quad::colored(
                whole * 0.5,
                whole,
                vec4(FLASH.x, FLASH.y, FLASH.z, self.flash.alpha()),
            ));
        }

        // the meter, along the bottom
        if !self.across {
            let middle = self.width * 0.5;
            let up = self.height - METER_TALL - METER_MARGIN;

            geometry.push_quad(&Quad::colored(
                vec2(middle, up),
                vec2(METER_WIDE, METER_TALL),
                METER_EMPTY,
            ));

            // the part that fills grows from the left edge of the track, so
            // its middle moves as it fills
            let filled = METER_WIDE * self.meter.filled();
            if filled > 0.0 {
                geometry.push_quad(&Quad::colored(
                    vec2(middle - METER_WIDE * 0.5 + filled * 0.5, up),
                    vec2(filled, METER_TALL),
                    METER_FULL,
                ));
            }
        }
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        let (Some(ground), Some(crates), Some(wall), Some(finish), Some(wire)) = (
            self.ground,
            self.crates,
            self.wall,
            self.finish,
            self.wire_mesh,
        ) else {
            return;
        };

        // no sun. The beams are the scene.
        scene.light.intensity = 0.0;
        scene.light.ambient = Vec3::splat(AMBIENT);

        scene.push_colored(ground, &Transform::default(), GROUND);
        scene.push_colored(crates, &Transform::default(), CRATES);
        scene.push_colored(wall, &Transform::default(), WALL);
        scene.push_colored(finish, &Transform::default(), FINISH);
        scene.push_colored(wire, &Transform::default(), WIRE);

        for beam in self.lights() {
            scene.push_spot(beam);
        }

        camera.position = self.you.eye();
        camera.target = camera.position + self.you.facing();
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let down = input.state == KeyboardKeyState::Pressed;

        match input.key {
            KeyboardKey::W | KeyboardKey::Up => self.held[0] = down,
            KeyboardKey::S | KeyboardKey::Down => self.held[1] = down,
            KeyboardKey::A => self.held[2] = down,
            KeyboardKey::D => self.held[3] = down,
            KeyboardKey::Left => self.held[4] = down,
            KeyboardKey::Right => self.held[5] = down,
            _ => {}
        }

        if !down || input.repeat {
            return;
        }

        if input.key == KeyboardKey::Escape {
            if self.locked {
                self.wants_lock = false;
            } else {
                self.quitting = true;
            }
        }
    }

    fn process_mouse(&mut self, input: blitzkit::mouse::MouseInput) {
        if input.is_pressed() && !self.locked {
            self.wants_lock = true;
        }
    }

    fn mouse_motion(&mut self, delta: Vec2) {
        if self.locked {
            self.you.look(delta);
        }
    }

    fn is_quitting(&self) -> bool {
        self.quitting
    }

    fn focus_changed(&mut self, focus: bool) {
        if !focus {
            self.wants_lock = false;
            self.held = [false; 6];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn you_start_at_the_near_side() {
        let game = SweepGame::new();

        assert_eq!(game.you.position, yard::start());
        assert!(!game.across);
    }

    #[test]
    fn being_caught_puts_you_back() {
        let mut game = SweepGame::new();
        game.you.position = Vec3::ZERO;

        game.send_back();

        assert_eq!(game.you.position, yard::start());
    }

    #[test]
    fn being_caught_keeps_you_facing_the_same_way() {
        // turning you round as well would be a second punishment for one mistake
        let mut game = SweepGame::new();
        game.you.yaw = 1.2;

        game.send_back();

        assert_eq!(game.you.yaw, 1.2);
    }

    #[test]
    fn a_wire_sends_you_back_with_no_meter_to_fill() {
        // a beam gives you a moment and a wire gives you none
        let mut game = SweepGame::new();
        let under = game.wires[0].bounds.center();
        let on_it = glam::vec3(under.x, 0.0, under.z);

        game.you.position = on_it;
        assert!(wires::tripped(&game.wires, on_it, on_it));
        assert_eq!(game.meter.filled(), 0.0, "it took a meter to catch you");
    }

    #[test]
    fn there_is_one_light_per_beam() {
        let game = SweepGame::new();

        assert_eq!(game.lights().len(), game.beams.len());
        assert!(game.lights().len() <= beams::BEAMS);
    }
}
