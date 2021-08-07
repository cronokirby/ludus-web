import { Emulator } from "../Cargo.toml";

/**
 * The number of seconds in our samples
 */
const MIN_SAMPLE_DURATION = 1 / 120;
const audioCtx = new AudioContext();
const SAMPLE_RATE = audioCtx.sampleRate;


const minSampleSize = SAMPLE_RATE * MIN_SAMPLE_DURATION;

let samples = new Float32Array(minSampleSize);

let flushing = false;

function pushSamples(newSamples: Float32Array) {
  const out = new Float32Array(samples.length + newSamples.length);
  out.set(samples);
  out.set(newSamples, samples.length);
  samples = out;
  if (!flushing) {
      flushSamples()
  }
}

function flushSamples() {
  if (samples.length < minSampleSize) {
    flushing = false;
    return;
  }
  flushing = true;
  const buf = audioCtx.createBuffer(1, samples.length, SAMPLE_RATE);
  buf.copyToChannel(samples, 0);
  samples = new Float32Array(0);
  const node = audioCtx.createBufferSource();
  node.buffer = buf;
  node.onended = flushSamples;
  node.connect(audioCtx.destination);
  node.start(0);
}

const emu = new Emulator(SAMPLE_RATE);

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

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");

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
  const buffer = emu.step(ctx, diff);
  pushSamples(buffer);

  window.requestAnimationFrame(loop);
}

window.requestAnimationFrame(loop);
