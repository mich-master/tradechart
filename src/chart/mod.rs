use core::slice;
use std::ops::{ Range, RangeBounds, Bound };
use chrono::{ DateTime, Utc, TimeZone, NaiveDateTime, NaiveDate, NaiveTime };
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext, WebGlProgram, WebGlUniformLocation};

pub mod tradedata;
mod shaders;

use crate::moex;
use tradedata::{ Hlocv, TradeData, TradeItemPositioned, union };

const DEFAULT_CANDLE_INTERVAL: u32 = 12;
const DEFAULT_CANDLE_RADIUS: u32 = 4;

#[derive(Clone)]
struct Period<Tz: TimeZone> {
    b: DateTime<Tz>,
    e: DateTime<Tz>,
}

impl Default for Period<Utc> {
    fn default() -> Self {
        Period { b: DateTime::<Utc>::MIN_UTC, e: DateTime::<Utc>::MIN_UTC }
    }
}

impl Period<Utc> {
    fn consider(&mut self, d: DateTime<Utc>) {
        if self.b > d || self.b == DateTime::<Utc>::MIN_UTC { self.b = d }
        if self.e < d { self.e = d }
    }
}


#[derive(Debug)]
#[derive(Clone, Default)]
#[derive(PartialEq)]
pub struct RangeF32(Range<f32>);

impl From<Range<f32>> for RangeF32 {
    fn from(v: Range<f32>) -> RangeF32 {
        RangeF32(v)
    }
}

impl RangeBounds<f32> for RangeF32 {
    fn start_bound(&self) -> Bound<&f32> {
        self.0.start_bound()
    }
    fn end_bound(&self) -> Bound<&f32> {
        self.0.end_bound()
    }
}

impl RangeF32 {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn new_with_max_rev() -> RangeF32 {
        RangeF32::from(f32::MAX..f32::MIN)
    }
    pub fn consider(&mut self, l: f32, h: f32) {
        if self.0.start > l { self.0.start = l; }
        if self.0.end < h { self.0.end = h; }
    }
    pub fn shift(&mut self, step: f32) {
        self.0.start += step;
        self.0.end += step;
    }
    pub fn set_width_from_high(&mut self, width: f32) {
        self.0.start = self.0.end - width;
    }
    pub fn start(&self) -> f32 {
        self.0.start
    }
    pub fn end(&self) -> f32 {
        self.0.end
    }
    pub fn size(&self) -> Option<f32> {
        if !self.is_empty() { Some(self.0.end - self.0.start) } else { None }
    }
    pub fn grid_start_step(&self) -> Option<(f32,f32)> {
        self.size().map(|size| {
            let l10: f32 = size.log10();
            let l10_round_1: f32 = l10.floor() - 1.0;
            let l10_fract: f32 = l10.fract();
            let base: f32 = 10.0;
            let bf: f32 = base.powf(l10_fract);
            let multiplier: f32 =
                if bf < 2.0 {
                    1.0
                } else if bf < 5.0 {
                    2.0
                } else {
                    5.0
                };
            let grid_step: f32 = base.powf(l10_round_1) * multiplier;
            let grid_start: f32 = (self.0.start / grid_step).floor() * grid_step;
            (grid_start, grid_step)
        })

    }
}


#[derive(Debug)]
#[derive(Clone, Default)]
#[derive(PartialEq)]
pub struct Frame(RangeF32,RangeF32);

impl Frame {
    pub fn new<T>(x: T, y: T) -> Frame
    where
        RangeF32: From<T>,
    {
        Frame (RangeF32::from(x), RangeF32::from(y))
    }
    pub fn range_x(&self) -> &RangeF32 {
        &self.0
    }
    pub fn range_x_mut(&mut self) -> &mut RangeF32 {
        &mut self.0
    }
    pub fn range_y(&self) -> &RangeF32 {
        &self.1
    }
    pub fn _range_y_mut(&mut self) -> &mut RangeF32 {
        &mut self.1
    }
    pub fn width(&self) -> Option<f32> {
        self.0.size()
    }
    pub fn height(&self) -> Option<f32> {
        self.1.size()
    }
}


pub enum TradeInterval {
    Day,
    // Hour,
}

impl TradeInterval {
    pub fn _seconds(&self) -> u32 {
        match self {
            Self::Day   => 86400,
            // Self::Hour  => 3600,
        }
    }
}

pub struct CandleOptions {
    interval: u32,
    radius: u32,
}

impl Default for CandleOptions {
    fn default() -> Self {
        CandleOptions {
            interval: DEFAULT_CANDLE_INTERVAL,
            radius: DEFAULT_CANDLE_RADIUS,
        }
    }
}

#[repr(C)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
}

impl Default for Point {
    fn default() -> Point {
        Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}


#[derive(Clone)]
#[repr(C)]
pub struct WebGlColor {
    r: f32,
    g: f32,
    b: f32,
}

struct WebGlIndexes {
    lines: Vec<u16>,
    triangles: Vec<u16>,
}

pub struct ChartGlData {
    points: Vec<Point>,
    colors: Vec<WebGlColor>,
    indexes: WebGlIndexes,
    frame: Frame,
    _interval: TradeInterval,
    candle_options: CandleOptions,
}

impl ChartGlData {
    pub fn new() -> ChartGlData {
        ChartGlData {
            points: Vec::new(),
            colors: Vec::new(),
            indexes: WebGlIndexes {
                lines: Vec::new(),
                triangles: Vec::new(),
            },
            frame: Frame::default(),
            _interval: TradeInterval::Day,
            candle_options: CandleOptions::default(),
        }
    }
    pub fn from_trade_data(trade_data: TradeData, candle_options: CandleOptions) -> ChartGlData {
        let frame: Frame = Frame::new(
            RangeF32::from(0.0..(trade_data.len() as u32 * candle_options.interval) as f32),
            trade_data.range().clone(),
        );

        let mut data: ChartGlData =
            ChartGlData {
                points: Vec::new(),
                colors: Vec::new(),
                indexes: WebGlIndexes {
                    lines: Vec::new(),
                    triangles: Vec::new(),
                },
                frame,
                _interval: TradeInterval::Day,
                candle_options,
            };

        trade_data.visualize(&mut data);

        data
    }
}

struct ChartGlView {
    canvas:  HtmlCanvasElement,
    context: WebGlRenderingContext,
    program: WebGlProgram,
    scale_uniform: Option<WebGlUniformLocation>,
    translation_uniform: Option<WebGlUniformLocation>,
    frame: Frame,
    canvas_size: (u32, u32),
}

impl ChartGlView {
    fn new() -> Result<ChartGlView, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();

        let canvas: web_sys::HtmlCanvasElement = document
            .get_element_by_id("chart")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
    
        let context: web_sys::WebGlRenderingContext = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        let program: WebGlProgram = shaders::make_shader_program(&context)?;
        context.use_program(Some(&program));

        let scale_uniform = context.get_uniform_location(&program, "scale");
        let translation_uniform = context.get_uniform_location(&program, "translation");

        let canvas_size = (canvas.width(),canvas.height());
        let frame: Frame = Frame::new( 0.0..canvas_size.0 as f32, 0.0..canvas_size.1 as f32);
        Ok(
            ChartGlView {
                canvas,
                context,
                program,
                scale_uniform,
                translation_uniform,
                frame,
                canvas_size,
            }
        )
    }
    fn buffer_points(&self, points: &[Point]) -> Result<(), JsValue> {
        let buffer = self.context.create_buffer().ok_or("failed to create buffer")?;
        self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let point_slice: &[f32] = slice::from_raw_parts(points.as_ptr() as *const _, points.len() * 3);
            let point_array = js_sys::Float32Array::view(point_slice);

            self.context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &point_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        let position_attribute: u32 = self.context.get_attrib_location(&self.program, "position") as u32;
        self.context.vertex_attrib_pointer_with_i32(position_attribute, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        self.context.enable_vertex_attrib_array(position_attribute);

        Ok(())
    }
    fn buffer_colors(&self, colors: &[WebGlColor]) -> Result<(), JsValue> {
        let buffer_colors = self.context.create_buffer().ok_or("failed to create buffer")?;
        self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer_colors));
    
        unsafe {
            let color_slice: &[f32] = slice::from_raw_parts(colors.as_ptr() as *const _, colors.len() * 3);
            let color_array = js_sys::Float32Array::view(color_slice);
    
            self.context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &color_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }
    
        let color_attribute: u32 = self.context.get_attrib_location(&self.program, "color") as u32;
        self.context.vertex_attrib_pointer_with_i32(color_attribute, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        self.context.enable_vertex_attrib_array(color_attribute);

        Ok(())
    }

    fn draw_indices(&self, indices: &[u16], item_type: u32) -> Result<(), JsValue> {
        let buffer_indices = self.context.create_buffer().ok_or("failed to create buffer")?;
        self.context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer_indices));

        unsafe {
            let indices_array = js_sys::Uint16Array::view(indices);
    
            self.context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &indices_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        };

        self.context.draw_elements_with_i32(
            item_type,
            indices.len() as i32,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );

        Ok(())
    }

    fn buffer_data(&self, points: &[Point], colors: &[WebGlColor]) -> Result<(), JsValue> {
        self.buffer_points(points)?;
        self.buffer_colors(colors)
    }

    fn draw_data(&self, indices: &WebGlIndexes) -> Result<(), JsValue> {
        self.context.clear_color(0.9, 0.9, 0.9, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.context.uniform2f(self.translation_uniform.as_ref(), self.frame.range_x().start(), self.frame.range_y().start());
        self.context.uniform2f(self.scale_uniform.as_ref(), 2.0 / self.frame.width().unwrap(), 2.0 / self.frame.height().unwrap());

        self.draw_indices(&indices.lines, WebGlRenderingContext::LINES)?;
        self.draw_indices(&indices.triangles, WebGlRenderingContext::TRIANGLES)
    }

    fn adjust_viewport(&mut self) -> Result<(), JsValue> {
        let eq_width: bool = self.canvas_size.0 != self.canvas.width();
        let eq_height: bool = self.canvas_size.1 != self.canvas.height();
        if eq_width || eq_height {
            if eq_width {
                self.canvas_size.0 = self.canvas.width();
                self.frame.range_x_mut().set_width_from_high(self.canvas_size.0 as f32);
            }
            if eq_height {
                self.canvas_size.1 = self.canvas.height();
            }
            self.context.viewport(0,0,self.canvas_size.0 as i32, self.canvas_size.1 as i32);
        };
        Ok(())
    }
}

#[wasm_bindgen]
pub struct TradeChart {
    data: ChartGlData,
    view: ChartGlView,
}

#[wasm_bindgen]
impl TradeChart {
    pub fn new() -> Result<TradeChart, JsValue> {
        Ok(
            TradeChart {
                data: ChartGlData::new(),
                view: ChartGlView::new()?,
            }
        )
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        self.view.adjust_viewport()?;
        self.view.draw_data(&self.data.indexes)
    }

    pub async fn display(&mut self, ticker: &str) -> Result<(), JsValue> {

        // let js: JsValue = ticker.into();
        // web_sys::console::log_2(&"ticker = ".into(), &js);

        let date_from: NaiveDateTime = NaiveDateTime::new(NaiveDate::from_ymd_opt(2022, 12, 1).unwrap(), NaiveTime::default());
        let trade_data: TradeData = moex::Moex::request_data(ticker, date_from).await.unwrap();
        let mut data = ChartGlData::from_trade_data(trade_data, CandleOptions::default());

        let extra_space_y: f32 = data.frame.height().unwrap() * 0.5; 
        self.view.frame = Frame::new(
            data.frame.range_x().end() - self.view.canvas.width() as f32 .. data.frame.range_x().end(),
            data.frame.range_y().start() - extra_space_y .. data.frame.range_y().end() + extra_space_y,
        );

        union(&data.frame, &self.view.frame).visualize(&mut data);

        self.data = data;

        self.view.buffer_data(&self.data.points, &self.data.colors)?;

        self.draw()

    }

    pub fn shift(&mut self, x: f32) -> Result<(), JsValue> {

        self.view.frame.range_x_mut().shift(x * self.data.candle_options.interval as f32);

        self.draw()
    }
}


trait Visualize {
    fn visualize(&self, data: &mut ChartGlData);
}

impl Visualize for Frame {
    fn visualize(&self, data: &mut ChartGlData) {
        let line_color = WebGlColor { r: 0.99, g: 0.99, b: 0.99 };
        let z: f32 = -0.1;

        let (mut y, grid_step) = self.range_y().grid_start_step().unwrap();
        while y < self.range_y().end() {

            data.indexes.lines.push( data.points.len() as u16 );
            data.points.push( Point {x: data.frame.range_x().start(), y, z } );
            data.colors.push( line_color.clone() );

            data.indexes.lines.push( data.points.len() as u16 );
            data.points.push( Point {x: data.frame.range_x().end(), y, z } );
            data.colors.push( line_color.clone() );

            y += grid_step;
        }
    }
}

impl Visualize for TradeItemPositioned<'_> {
    fn visualize(&self, data: &mut ChartGlData) {
        let x: f32 = self.position as f32;
        let width: f32 = data.candle_options.radius as f32;
        let z: f32 = 0.0;

        let hlocv: &Hlocv = self.item.hlocv();
        if hlocv.o == hlocv.c {
            let green_candle_color = WebGlColor { r: 0.1, g: 0.6, b: 0.1 };

            data.indexes.lines.push( data.points.len() as u16 );
            data.points.push( Point {x, y: hlocv.h, z } );
            data.colors.push( green_candle_color.clone() );

            data.indexes.lines.push( data.points.len() as u16 );
            data.points.push( Point {x, y: hlocv.l, z } );
            data.colors.push( green_candle_color.clone() );

            data.indexes.lines.push( data.points.len() as u16 );
            data.points.push( Point {x: x-width, y: hlocv.o, z } );
            data.colors.push( green_candle_color.clone() );

            data.indexes.lines.push( data.points.len() as u16 );
            data.points.push( Point {x: x+width, y: hlocv.c, z } );
            data.colors.push( green_candle_color );
        } else {
            let (body_high, body_low, candle_color): (f32, f32, WebGlColor) =
                if hlocv.o > hlocv.c {
                    (hlocv.o, hlocv.c, WebGlColor { r: 0.9, g: 0.1, b: 0.1 } )
                } else {
                    (hlocv.c, hlocv.o, WebGlColor { r: 0.1, g: 0.6, b: 0.1 } )
                };

            if hlocv.h > body_high || hlocv.l < body_low {
                let y1: f32 = 
                    if hlocv.h > body_high {
                        hlocv.h
                    } else {
                        body_low
                    };
                let y2: f32 = 
                    if hlocv.l < body_low {
                        hlocv.l
                    } else {
                        body_high
                    };
                
                data.indexes.lines.push( data.points.len() as u16 );
                data.points.push( Point { x, y: y1, z } );
                data.colors.push( candle_color.clone() );

                data.indexes.lines.push( data.points.len() as u16 );
                data.points.push( Point { x, y: y2, z } );
                data.colors.push( candle_color.clone() );
            }

            data.indexes.triangles.push( data.points.len() as u16 );
            data.points.push( Point {x: x-width, y: hlocv.o, z } );
            data.colors.push( candle_color.clone() );

            data.indexes.triangles.push( data.points.len() as u16 );
            data.points.push( Point {x: x+width, y: hlocv.o, z } );
            data.colors.push( candle_color.clone() );

            data.indexes.triangles.push( data.points.len() as u16 );
            data.points.push( Point {x: x+width, y: hlocv.c, z } );
            data.colors.push( candle_color.clone() );

            let idx: u16 = data.points.len() as u16;
            data.indexes.triangles.push( idx-3 );
            data.indexes.triangles.push( idx-1 );
            data.indexes.triangles.push( idx );
            data.points.push( Point {x: x-width, y: hlocv.c, z } );
            data.colors.push( candle_color );
        }
    }
}

impl Visualize for TradeData {
    fn visualize(&self, data: &mut ChartGlData) {
        for (i,item) in self.iter_data().enumerate() {
            TradeItemPositioned::new(&item, i as u32 * data.candle_options.interval).visualize(data);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::shapes::Range_f32;

//     #[test]
//     fn step_calculation() {
//         let y2: f32 = 60.0;
//         // let l10: f32 = y2.log10().round() - 1.0;
//         // assert_eq!(l10, 1.0);

//         println!("-: {}, {}, {}", y2.log10(), y2.log10().round(), y2.log10().fract());
//         let base: f32 = 10.0;
//         println!("-: {}", base.powf(y2.log10().fract()));
//         // let y2: f32 = 10.0;
//         println!("{}: {}", y2, base.powf(y2.log10().fract()));
//         let y2: f32 = 40.0;
//         println!("40: {}", base.powf(y2.log10().fract()));
//         let y2: f32 = 70.0;
//         println!("70: {}", base.powf(y2.log10().fract()));
//         let y2: f32 = 90.0;
//         println!("90: {}", base.powf(y2.log10().fract()));
//         let y2: f32 = 120.0;
//         println!("120: {}", base.powf(y2.log10().fract()));
//         // assert_eq!(calc_step_y( &Range::new(100.0, 200.0) ), 2.0);
//         assert_eq!(calc_step_y( &Range::new(100.0, 200.0) ), 10.0);
//         assert_eq!(calc_step_y( &Range::new(10.0, 20.0) ), 1.0);
//         assert_eq!(calc_step_y( &Range::new(1.0, 2.0) ), 0.1);
//         assert_eq!(calc_step_y( &Range::new(0.1, 0.2) ), 0.01);
//         assert_eq!(calc_step_y( &Range::new(0.1, 200.0) ), 10.0);
//         // assert_eq!(calc_step_y( &Range::new(180.0, 200.0) ), 1.0);
//     }
// }
