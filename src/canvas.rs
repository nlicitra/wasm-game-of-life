use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::*;

pub fn initialize_canvas_context() -> Result<CanvasRenderingContext2d, JsValue> {
  let window = window().unwrap();
  let document = window.document().unwrap();
  let canvas = document.get_element_by_id("game-of-life-canvas").unwrap();
  let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
  let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?.unwrap().dyn_into()?;

  Ok(ctx)
}
