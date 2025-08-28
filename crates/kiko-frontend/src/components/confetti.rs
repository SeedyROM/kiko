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
}

impl ConfettiParticle {
    fn new(width: f64) -> Self {
        let colors = [
            "#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#feca57", "#ff9ff3", "#54a0ff", "#5f27cd",
            "#00d2d3", "#ff9f43",
        ];
        let color_index = (Math::random() * colors.len() as f64) as usize;

        Self {
            x: Math::random() * width,
            y: -20.0,
            vx: (Math::random() - 0.5) * 10.0,
            vy: Math::random() * -15.0 - 5.0,
            gravity: 0.3,
            color: colors[color_index].to_string(),
            size: Math::random() * 8.0 + 4.0,
            rotation: Math::random() * 360.0,
            rotation_speed: (Math::random() - 0.5) * 10.0,
        }
    }

    fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += self.gravity;
        self.rotation += self.rotation_speed;
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        ctx.save();
        ctx.translate(self.x, self.y).unwrap();
        ctx.rotate(self.rotation * std::f64::consts::PI / 180.0)
            .unwrap();
        #[allow(warnings)]
        ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(&self.color));
        ctx.fill_rect(-self.size / 2.0, -self.size / 2.0, self.size, self.size);
        ctx.restore();
    }

    fn is_off_screen(&self, canvas_height: f64) -> bool {
        self.y > canvas_height + 50.0
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

                for _ in 0..50 {
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
                                particle.update();
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
