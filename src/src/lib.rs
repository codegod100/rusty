use sauron::{
    html::text, html::units::px, jss, node, wasm_bindgen, Application, Cmd, Node, Program,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, window};

// Bind to Tauri's invoke function
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    fn requestAnimationFrame(callback: &Closure<dyn FnMut(f64)>) -> i32;
    fn cancelAnimationFrame(id: i32);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

enum Msg {
    Increment,
    Decrement,
    Reset,
    GetRandomData,
    RandomDataResponse(String),
    ToggleAnimation,
    AnimationFrame(f64),
}

struct App {
    count: i32,
    random_data: String,
    loading: bool,
    animation_running: bool,
    animation_time: f64,
}

impl App {
    fn new() -> Self {
        App { 
            count: 0,
            random_data: String::new(),
            loading: false,
            animation_running: false,
            animation_time: 0.0,
        }
    }
}

fn get_canvas() -> Option<HtmlCanvasElement> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let canvas = document.get_element_by_id("myCanvas")?;
    canvas.dyn_into::<HtmlCanvasElement>().ok()
}

fn get_context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()?
        .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
}

fn start_animation_loop() -> Cmd<Msg> {
    Cmd::new(async {
        // Start with current timestamp
        Msg::AnimationFrame(js_sys::Date::now())
    })
}

fn continue_animation_loop() -> Cmd<Msg> {
    Cmd::new(async {
        // Simple delay using a Promise-based timeout
        let window = web_sys::window().unwrap();
        
        // Create a simple promise that resolves after 16ms (~60fps)
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let closure = Closure::wrap(Box::new(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut()>);
            
            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(), 16
            ).unwrap();
            closure.forget();
        });
        
        // Wait for the promise and then return the next frame
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
        Msg::AnimationFrame(js_sys::Date::now())
    })
}

fn animate_canvas(canvas: &HtmlCanvasElement, time: f64, count: i32) {
    if let Some(ctx) = get_context(canvas) {
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        
        // Clear canvas with gradient background
        clear_canvas(canvas);
        
        // Animation parameters based on time and count
        let t = time * 0.001; // Convert to seconds
        let speed_multiplier = (count.abs() as f64 * 0.1 + 1.0).min(3.0);
        
        // Draw bouncing balls
        for i in 0..3 {
            let phase = i as f64 * 2.1;
            let ball_speed = speed_multiplier * (1.0 + i as f64 * 0.3);
            
            // Bouncing motion
            let x = (width * 0.5) + ((t * ball_speed + phase).sin() * (width * 0.3));
            let y = (height * 0.5) + ((t * ball_speed * 1.5 + phase).cos() * (height * 0.25));
            
            // Color that changes over time
            let hue = ((t * 50.0 + i as f64 * 120.0) % 360.0) as i32;
            let color = format!("hsl({}, 70%, 60%)", hue);
            
            ctx.set_fill_style(&JsValue::from_str(&color));
            ctx.begin_path();
            ctx.arc(x, y, 8.0 + (t + phase).sin().abs() * 5.0, 0.0, 2.0 * std::f64::consts::PI).unwrap();
            ctx.fill();
        }
        
        // Draw rotating squares
        for i in 0..2 {
            let phase = i as f64 * std::f64::consts::PI;
            let rotation_speed = speed_multiplier * (0.5 + i as f64 * 0.2);
            
            let cx = width * (0.25 + i as f64 * 0.5);
            let cy = height * 0.5;
            let size = 20.0 + (t * 2.0 + phase).sin().abs() * 10.0;
            
            ctx.save();
            ctx.translate(cx, cy).unwrap();
            ctx.rotate(t * rotation_speed).unwrap();
            
            let hue = ((t * 30.0 + i as f64 * 180.0) % 360.0) as i32;
            let color = format!("hsl({}, 80%, 50%)", hue);
            ctx.set_fill_style(&JsValue::from_str(&color));
            ctx.fill_rect(-size / 2.0, -size / 2.0, size, size);
            
            ctx.restore();
        }
        
        // Draw spiral pattern
        ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 255, 0.6)"));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        
        let center_x = width * 0.5;
        let center_y = height * 0.5;
        let spiral_speed = speed_multiplier * 0.3;
        
        for i in 0..50 {
            let angle = i as f64 * 0.3 + t * spiral_speed;
            let radius = i as f64 * 2.0;
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;
            
            if i == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.stroke();
        
        // Draw pulsing circle in center
        let pulse_radius = 15.0 + (t * 4.0).sin().abs() * 10.0;
        let alpha = 0.3 + (t * 3.0).sin().abs() * 0.4;
        let pulse_color = format!("rgba(255, 255, 255, {})", alpha);
        
        ctx.set_fill_style(&JsValue::from_str(&pulse_color));
        ctx.begin_path();
        ctx.arc(center_x, center_y, pulse_radius, 0.0, 2.0 * std::f64::consts::PI).unwrap();
        ctx.fill();
    }
}

fn clear_canvas(canvas: &HtmlCanvasElement) {
    if let Some(ctx) = get_context(canvas) {
        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        
        // Add a subtle animated background gradient
        let gradient = ctx.create_radial_gradient(
            canvas.width() as f64 * 0.5, 
            canvas.height() as f64 * 0.5, 
            0.0,
            canvas.width() as f64 * 0.5, 
            canvas.height() as f64 * 0.5, 
            canvas.width() as f64 * 0.7
        ).unwrap();
        
        gradient.add_color_stop(0.0, "rgba(20, 30, 48, 0.9)").unwrap();
        gradient.add_color_stop(1.0, "rgba(36, 59, 85, 0.7)").unwrap();
        ctx.set_fill_style(&JsValue::from(gradient));
        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
}

impl Application for App {

    type MSG = Msg;

    fn view(&self) -> Node<Msg> {
        let animation_running = self.animation_running;
        let loading = self.loading;
        
        node! {
            <main>
                <div class="app-title">
                    {text("Rusty Counter & Canvas App")}
                </div>
                
                <div class="counter-section">
                    <button class="btn btn-decrement" on_click=|_| Msg::Decrement>
                        {text("−")}
                    </button>
                    <div class="count-display" on_click=|_| Msg::Reset>
                        {text(self.count)}
                    </div>
                    <button class="btn btn-increment" on_click=|_| Msg::Increment>
                        {text("+")}
                    </button>
                </div>
                
                <div class="reset-hint">
                    {text("Click the number to reset")}
                </div>
                
                <div class="canvas-section">
                    <canvas id="myCanvas" width="300" height="200" class="canvas">
                    </canvas>
                    <div class="canvas-controls">
                        <button class="btn btn-canvas" on_click=|_| Msg::ToggleAnimation>
                            {text(if animation_running { "⏸️ Stop Animation" } else { "▶️ Start Animation" })}
                        </button>
                    </div>
                </div>
                
                <div class="api-section">
                    <button class="btn btn-api" on_click=move |_| Msg::GetRandomData>
                        {text(if loading { "🔄 Loading..." } else { "🎲 Get Random Data" })}
                    </button>
                    <div class="random-data-display">
                        {text(&self.random_data)}
                    </div>
                </div>
            </main>
        }
    }

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::Decrement => self.count -= 1,
            Msg::Reset => self.count = 0,
            Msg::GetRandomData => {
                self.loading = true;
                self.random_data = "Loading...".to_string();
                // Call Tauri invoke command for random data
                return Cmd::new(async {
                    let args = js_sys::Object::new();
                    
                    match invoke("get_random_data", args.into()).await {
                        result => {
                            let response = result.as_string().unwrap_or_else(|| "Error fetching data".to_string());
                            Msg::RandomDataResponse(response)
                        }
                    }
                });
            },
            Msg::RandomDataResponse(data) => {
                self.loading = false;
                self.random_data = data;
            },
            Msg::ToggleAnimation => {
                if self.animation_running {
                    console_log!("Stopping animation");
                    self.animation_running = false;
                    if let Some(canvas) = get_canvas() {
                        clear_canvas(&canvas);
                    }
                } else {
                    console_log!("Starting animation");
                    self.animation_running = true;
                    self.animation_time = 0.0;
                    if let Some(canvas) = get_canvas() {
                        clear_canvas(&canvas);
                    }
                    return start_animation_loop();
                }
            },
            Msg::AnimationFrame(time) => {
                if self.animation_running {
                    self.animation_time = time;
                    if let Some(canvas) = get_canvas() {
                        animate_canvas(&canvas, time, self.count);
                    }
                    return continue_animation_loop();
                } else {
                    // Animation was stopped, make sure canvas is cleared
                    if let Some(canvas) = get_canvas() {
                        clear_canvas(&canvas);
                    }
                }
            }
        }
        Cmd::none()
    }

    fn stylesheet() -> Vec<String> {
        vec![jss! {
            "body": {
                font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
                margin: "0",
                padding: "20px",
                background: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
                min_height: "100vh",
                color: "#333",
            },

            "main": {
                max_width: px(400),
                margin: "0 auto",
                background: "rgba(255, 255, 255, 0.95)",
                border_radius: px(20),
                padding: px(40),
                box_shadow: "0 10px 30px rgba(0, 0, 0, 0.2)",
                text_align: "center",
                backdrop_filter: "blur(10px)",
            },

            ".app-title": {
                font_size: px(28),
                font_weight: "700",
                color: "#4a5568",
                margin_bottom: px(30),
                text_shadow: "0 2px 4px rgba(0, 0, 0, 0.1)",
            },

            ".counter-section": {
                display: "flex",
                align_items: "center",
                justify_content: "center",
                gap: px(20),
                margin_bottom: px(15),
            },

            ".btn": {
                border: "none",
                border_radius: px(12),
                cursor: "pointer",
                font_weight: "600",
                transition: "all 0.2s ease",
                outline: "none",
                box_shadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
            },

            ".btn:hover": {
                transform: "translateY(-2px)",
                box_shadow: "0 6px 16px rgba(0, 0, 0, 0.2)",
            },

            ".btn:active": {
                transform: "translateY(0)",
                box_shadow: "0 2px 8px rgba(0, 0, 0, 0.15)",
            },

            ".btn-increment, .btn-decrement": {
                width: px(60),
                height: px(60),
                font_size: px(24),
                color: "white",
            },

            ".btn-increment": {
                background: "linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)",
            },

            ".btn-decrement": {
                background: "linear-gradient(135deg, #fa709a 0%, #fee140 100%)",
            },

            ".count-display": {
                font_size: px(48),
                font_weight: "800",
                color: "#2d3748",
                min_width: px(100),
                padding: px(20),
                background: "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)",
                background_clip: "text",
                "-webkit-background-clip": "text",
                "-webkit-text-fill-color": "transparent",
                cursor: "pointer",
                user_select: "none",
                transition: "all 0.2s ease",
            },

            ".count-display:hover": {
                transform: "scale(1.05)",
            },

            ".reset-hint": {
                font_size: px(12),
                color: "#718096",
                margin_bottom: px(25),
                font_style: "italic",
            },

            ".canvas-section": {
                border_top: "1px solid #e2e8f0",
                padding_top: px(25),
                margin_bottom: px(25),
            },

            ".canvas": {
                border: "2px solid #4a5568",
                border_radius: px(12),
                background: "radial-gradient(circle, #1a202c 0%, #2d3748 100%)",
                box_shadow: "inset 0 2px 8px rgba(0, 0, 0, 0.3), 0 4px 16px rgba(0, 0, 0, 0.2)",
                margin_bottom: px(15),
            },

            ".canvas-controls": {
                display: "flex",
                gap: px(10),
                justify_content: "center",
                flex_wrap: "wrap",
            },

            ".btn-canvas": {
                background: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
                color: "white",
                font_size: px(16),
                padding: "12px 20px",
            },

            ".api-section": {
                border_top: "1px solid #e2e8f0",
                padding_top: px(25),
            },

            ".btn-api": {
                background: "linear-gradient(135deg, #11998e 0%, #38ef7d 100%)",
                color: "white",
                font_size: px(16),
                padding: "12px 24px",
                margin_bottom: px(15),
            },

            ".random-data-display": {
                min_height: px(30),
                padding: px(15),
                background: "#f7fafc",
                border_radius: px(8),
                color: "#4a5568",
                font_weight: "500",
                border: "1px solid #e2e8f0",
                margin_bottom: px(10),
                line_height: "1.5",
                word_wrap: "break-word",
            },
        }]
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    Program::mount_to_body(App::new());
}