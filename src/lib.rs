mod utils;

use std::ops::Index;
use js_sys::Array;
use js_sys::Map;
use web_sys::CanvasRenderingContext2d;
use web_sys::Request;
use web_sys::RequestInit;
use web_sys::Response;
use std::f64;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use futures::executor::block_on;


extern crate web_sys;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, vector2d-test!");
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ADot {
    id: i32,
    x: i32,
    y: i32,
}

#[wasm_bindgen]
impl ADot {
    pub fn is_near(&self, x: u32, y: u32) -> bool {
        
        let neighboor = 7;
        if (self.x - x as i32).abs() < neighboor || (self.y - y as i32).abs() < neighboor {
            // log!("is near <{}, {}>? ?? ({}, {})", self.x, self.y, x, y);
            true
        } else {
            false
        }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    dots: Vec<ADot>,
    mouse_down: bool,
    grabing: Option<i32>
}

#[wasm_bindgen]
impl Universe {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn dots(&self) -> *const ADot {
        self.dots.as_ptr()
    }

    pub fn dots_array(&self) -> Array {
        self.dots.clone().into_iter().map(JsValue::from).collect()
    }

    pub fn dots_map(&self) -> Map {
        let map = Map::new();
        let mut idx = 0;
        for d in self.dots.iter() {
            let m = Map::new();
            m.set(&"x".into(), &JsValue::from(d.x));
            m.set(&"y".into(), &JsValue::from(d.y));
            map.set(&format!("{}", idx).into(), &m);
            idx += 1;
        }
        map
    }

    async fn load_dots_for_user(&self, id: i32) -> Result<JsValue, JsValue>{
        let mut opts = RequestInit::new();
        opts.method("GET");

        let url = format!("http://localhost:8000/users/{}", id);

        let request = Request::new_with_str_and_init(&url, &opts)?;

        request
            .headers()
            .set("Accept", "application/vnd.github.v3+json")?;

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        let json = JsFuture::from(resp.json()?).await?;
        log!("{:?}", json);
        Ok(json)
    }

    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        
        let mouse_down = false;
        let grabing = None;
        // let dots:Vec<ADot> = vec!();

        let dots = vec![
            ADot{id: 2, x: 20, y: 20},
            ADot{id: 3, x: 30, y: 30},
            ADot{id: 4, x: 40, y: 40},
            ADot{id: 5, x: 50, y: 50}
        ];

        utils::set_panic_hook();
        let u = Universe {
            width,
            height,
            dots,
            mouse_down,
            grabing
        };
        // let r = block_on(u.load_dots_for_user(1));
        // match r {
        //     Ok(result) => {
        //         log!("{:?}", result);
        //         u
        //     },
        //     _ => u
        // }
        u
    }

    pub fn move_mouse(&mut self, e_x: u32, e_y: u32) {

        if let Some(id) = self.grabing {
            if self.mouse_down == false {
                self.grabing = None;
                return;
            }
            let x = self.dots.iter_mut().find(|&&mut d| d.id == id);
            let u: &mut ADot = x.unwrap();
            u.x = e_x as i32;
            u.y = e_y as i32;
        }
    }

    pub fn set_mouse_down(&mut self, x: u32, y: u32) {
        self.mouse_down = true;
        let x = self.dots.iter().find(|&&d| d.is_near(x, y));
        match x {
            Some(y) => {
                self.grabing = Some(y.id)
            },
            _ => {self.grabing = None}
        };
    }

    pub fn set_mouse_up(&mut self, _x: u32, _y: u32) {
        self.mouse_down = false;
        self.grabing = None;
    }

    pub fn render(&self) {
    }

    pub fn tick(&mut self) {
        let canvas = self.get_drawing_context();
        self.clear_bg(&canvas);
        self.draw_dots(&canvas);
    }

    pub fn dot_count(&self) -> usize {
        self.dots.len()
    }

    pub fn get_drawing_context(&self) -> CanvasRenderingContext2d {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("game-of-life-canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
    
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        context
    }

    pub fn clear_bg(&self, _ctx: &CanvasRenderingContext2d){
        let ctx = self.get_drawing_context();
        ctx
        .set_fill_style(&"rgb(238, 238, 238)".into());
        ctx.fill_rect(0.0, 0.0, 16000.0, 16000.0);
    }

    pub fn draw_dots(&self, context: &CanvasRenderingContext2d) {
        context.begin_path();
        let cell_size:f64 = 5.0;
        context
            .set_fill_style(&"rgb(240, 10, 10)".into());
        for d in self.dots.iter() {
            context.fill_rect(d.x as f64, d.y  as f64, cell_size, cell_size);
        }
        context.stroke()
    }
}

