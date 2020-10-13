import {Universe} from "wasm-game-of-life";
import Stats from "stats.js";

const size = 128;
const universe = Universe.new(size, size);

const stats = new Stats();
stats.showPanel(0); //  FPS
document.body.appendChild(stats.dom);
stats.dom.classList.add("stats");

const interval = 1000/60;
let timestamp = new Date().getTime();
const renderLoop = () => {
  stats.begin();
  //const now = new Date().getTime();
  //if (now - timestamp >= interval) {
    universe.tick();
    //timestamp = now;
    universe.render();
  //}
  stats.end();
  requestAnimationFrame(renderLoop);
}

requestAnimationFrame(renderLoop);
