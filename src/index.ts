import { Emulator } from "../Cargo.toml";

const audioCtx = new AudioContext();
const SAMPLE_RATE = 44100;

const emu = new Emulator(SAMPLE_RATE, audioCtx);

let settingsMenuOpen = false;

function openSettingsMenu() {
  if (settingsMenuOpen) {
    return;
  }
  settingsMenuOpen = true;
  const menu = document.getElementById("settings-menu-background");
  menu.classList.remove("invisible", "opacity-0");
  menu.classList.add("opacity-100");
}

function closeSettingsMenu() {
  if (!settingsMenuOpen) {
    return;
  }
  settingsMenuOpen = false;
  const menu = document.getElementById("settings-menu-background");
  menu.classList.remove("opacity-100");
  menu.classList.add("invisible", "opacity-0");
}

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
document.getElementById("settings-button").addEventListener("click", (e) => {
  openSettingsMenu();
  e.stopPropagation();
});
document.addEventListener("click", () => {
  if (settingsMenuOpen) {
    closeSettingsMenu();
  }
});
document.getElementById("settings-menu").addEventListener("click", (e) => {
  e.stopPropagation();
});
document.body.ondragover = document.body.ondragenter = (ev) => {
  ev.preventDefault();
};
document.body.addEventListener("drop", (ev) => {
  ev.preventDefault();
  let file = null as File | null;
  if (ev.dataTransfer.items) {
    // Use DataTransferItemList interface to access the file(s)
    for (var i = 0; i < ev.dataTransfer.items.length; i++) {
      // If dropped items aren't files, reject them
      if (ev.dataTransfer.items[i].kind === "file") {
        file = ev.dataTransfer.items[i].getAsFile();
        break;
      }
    }
  } else {
    file = ev.dataTransfer.files[0];
  }
  file.arrayBuffer().then((rom) => emu.swap_cart(new Uint8Array(rom)));
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
