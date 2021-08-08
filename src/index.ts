import { Emulator } from "../Cargo.toml";

const audioCtx = new AudioContext();
const SAMPLE_RATE = 44100;

const emu = new Emulator(SAMPLE_RATE, audioCtx);

const romSelector = document.getElementById("rom-selector") as HTMLInputElement;
romSelector.addEventListener(
  "change",
  function () {
    this.files
      .item(0)
      .arrayBuffer()
      .then((rom) => {
        emu.swap_cart(new Uint8Array(rom));
      });
  },
  false
);
document.getElementById("rom-selector-button").addEventListener("click", () => {
  romSelector.click();
});

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");

ctx.webkitImageSmoothingEnabled = false;
ctx.mozImageSmoothingEnabled = false;
ctx.imageSmoothingEnabled = false;

enum Buttons {
  A = 1,
  B = 1 << 1,
  Start = 1 << 2,
  Select = 1 << 3,
  Up = 1 << 4,
  Down = 1 << 5,
  Left = 1 << 6,
  Right = 1 << 7,
}

let buttons = 0;
const buttonmap = {
  j: Buttons.A,
  k: Buttons.B,
  h: Buttons.Start,
  g: Buttons.Select,
  w: Buttons.Up,
  s: Buttons.Down,
  a: Buttons.Left,
  d: Buttons.Right,
};

window.addEventListener("keydown", (ev) => {
  buttons |= buttonmap[ev.key] ?? 0;
  emu.update_buttons(buttons);
});
window.addEventListener("keyup", (ev) => {
  buttons &= ~(buttonmap[ev.key] ?? 0);
  emu.update_buttons(buttons);
});

let old = 0.0;

function loop(timestamp: number) {
  const diff = 1000 * (timestamp - old);
  old = timestamp;
  emu.step(ctx, diff);

  window.requestAnimationFrame(loop);
}

window.requestAnimationFrame(loop);
