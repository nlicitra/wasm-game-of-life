// import {gameOfLife} from './gol.js';


const {Universe, Cell, WorkerPool} = wasm_bindgen;

async function run() {
  const {memory} = await wasm_bindgen("pkg/wasm_game_of_life_bg.wasm");
  console.log(memory);
  gameOfLife(memory, Universe, Cell);
  const pool = new WorkerPool(navigator.hardwareConcurrency);
  console.log(pool);
}
run();
