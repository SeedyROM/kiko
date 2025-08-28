use gloo_timers::callback::Interval;
use js_sys::Math;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[derive(Clone)]
struct ConfettiParticle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    gravity: f64,
    color: String,
    size: f64,
    rotation: f64,
    rotation_speed: f64,
    shape: ConfettiShape,
    flutter_offset: f64,
    flutter_speed: f64,
    flutter_amplitude: f64,
    opacity: f64,
    time: f64,
}

#[derive(Clone, PartialEq)]
enum ConfettiShape {
    Square,
    Circle,
    Rectangle { width: f64, height: f64 },
    Line { length: f64 },
}

impl ConfettiParticle {
    fn new(width: f64) -> Self {
        let colors = [
            "#FF0080", "#00FF80", "#8000FF", "#FF8000", "#0080FF", "#80FF00", "#FF0040", "#40FF00",
            "#0040FF", "#FF4000", "#4000FF", "#00FF40", "#FF1493", "#00CED1", "#FFD700", "#FF69B4",
            "#00FA9A", "#FF6347", "#DA70D6", "#98FB98", "#F0E68C", "#FFA07A", "#20B2AA", "#87CEEB",
            "#DDA0DD", "#90EE90", "#FFFF54", "#FF7F50", "#6495ED", "#ADFF2F",
        ];
        let color_index = (Math::random() * colors.len() as f64) as usize;

        let shapes = [
            ConfettiShape::Square,
            ConfettiShape::Circle,
            ConfettiShape::Rectangle {
                width: Math::random() * 15.0 + 5.0,
                height: Math::random() * 8.0 + 3.0,
            },
            ConfettiShape::Line {
                length: Math::random() * 20.0 + 10.0,
            },
        ];
        let shape_index = (Math::random() * shapes.len() as f64) as usize;

        Self {
            x: Math::random() * width,
            y: -50.0,
            vx: (Math::random() - 0.5) * 16.0,
            vy: Math::random() * -20.0 - 8.0,
            gravity: Math::random() * 0.4 + 0.2,
            color: colors[color_index].to_string(),
            size: Math::random() * 12.0 + 6.0,
            rotation: Math::random() * 360.0,
            rotation_speed: (Math::random() - 0.5) * 15.0,
            shape: shapes[shape_index].clone(),
            flutter_offset: Math::random() * 360.0,
            flutter_speed: Math::random() * 3.0 + 1.0,
            flutter_amplitude: Math::random() * 30.0 + 10.0,
            opacity: 1.0,
            time: 0.0,
        }
    }

    fn update(&mut self, canvas_height: f64) {
        self.time += 0.16;

        // Add fluttering horizontal movement
        let flutter = (self.time * self.flutter_speed + self.flutter_offset).sin()
            * self.flutter_amplitude
            * 0.1;
        self.x += self.vx + flutter;
        self.y += self.vy;

        // Apply gravity with slight randomness
        self.vy += self.gravity;

        // Add air resistance to prevent particles from moving too fast
        self.vx *= 0.998;
        self.vy *= 0.999;

        // Update rotation with flutter effect
        self.rotation += self.rotation_speed + flutter * 0.5;

        // Only fade out particles that are near the bottom
        if self.y > canvas_height * 0.8 {
            let fade_progress = (self.y - canvas_height * 0.8) / (canvas_height * 0.2);
            self.opacity = 1.0 - fade_progress;
        }
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        ctx.save();
        ctx.translate(self.x, self.y).unwrap();
        ctx.rotate(self.rotation * std::f64::consts::PI / 180.0)
            .unwrap();

        // Set opacity for fading effect
        ctx.set_global_alpha(self.opacity);

        #[allow(warnings)]
        ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(&self.color));

        // Draw different shapes based on the particle's shape
        match &self.shape {
            ConfettiShape::Square => {
                ctx.fill_rect(-self.size / 2.0, -self.size / 2.0, self.size, self.size);
            }
            ConfettiShape::Circle => {
                ctx.begin_path();
                ctx.arc(0.0, 0.0, self.size / 2.0, 0.0, 2.0 * std::f64::consts::PI)
                    .unwrap();
                ctx.fill();
            }
            ConfettiShape::Rectangle { width, height } => {
                ctx.fill_rect(-width / 2.0, -height / 2.0, *width, *height);
            }
            ConfettiShape::Line { length } => {
                ctx.set_line_width(self.size / 3.0);
                #[allow(deprecated)]
                ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(&self.color));
                ctx.begin_path();
                ctx.move_to(-length / 2.0, 0.0);
                ctx.line_to(*length / 2.0, 0.0);
                ctx.stroke();
            }
        }

        ctx.restore();
    }

    fn is_off_screen(&self, canvas_height: f64) -> bool {
        self.y > canvas_height + 50.0 || self.opacity <= 0.0
    }
}

#[derive(Properties, PartialEq)]
pub struct ConfettiProps {
    pub trigger: Callback<Callback<()>>,
}

#[function_component(Confetti)]
pub fn confetti(props: &ConfettiProps) -> Html {
    let canvas_ref = use_node_ref();
    let particles = use_mut_ref(Vec::<ConfettiParticle>::new);
    let interval_handle = use_mut_ref(|| None::<Interval>);

    let trigger_confetti = {
        let particles = particles.clone();
        let canvas_ref = canvas_ref.clone();
        let interval_handle = interval_handle.clone();

        Callback::from(move |_| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let width = canvas.width() as f64;
                let mut particles_borrow = particles.borrow_mut();

                for _ in 0..256 {
                    particles_borrow.push(ConfettiParticle::new(width));
                }

                // Start animation if not already running
                if interval_handle.borrow().is_none() {
                    let particles_inner = particles.clone();
                    let canvas_ref_inner = canvas_ref.clone();
                    let interval_handle_inner = interval_handle.clone();

                    let interval = Interval::new(16, move || {
                        if let Some(canvas) = canvas_ref_inner.cast::<HtmlCanvasElement>() {
                            let ctx: CanvasRenderingContext2d = canvas
                                .get_context("2d")
                                .unwrap()
                                .unwrap()
                                .dyn_into()
                                .unwrap();

                            let width = canvas.width() as f64;
                            let height = canvas.height() as f64;

                            ctx.clear_rect(0.0, 0.0, width, height);

                            let mut particles_borrow = particles_inner.borrow_mut();

                            particles_borrow.retain_mut(|particle| {
                                particle.update(height);
                                particle.draw(&ctx);
                                !particle.is_off_screen(height)
                            });

                            if particles_borrow.is_empty() {
                                *interval_handle_inner.borrow_mut() = None;
                            }
                        }
                    });

                    *interval_handle.borrow_mut() = Some(interval);
                }
            }
        })
    };

    {
        let props_trigger = props.trigger.clone();
        let trigger = trigger_confetti.clone();
        use_effect_with((), move |_| {
            props_trigger.emit(trigger);
            || {}
        });
    }

    html! {
        <canvas
            ref={canvas_ref}
            width="1920"
            height="1080"
            style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; pointer-events: none; z-index: 9999; width: 100%; height: 100%;"
        />
    }
}
