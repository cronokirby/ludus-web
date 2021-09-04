import { Emulator } from "../Cargo.toml";

/**
 * Buttons maps each Button to a single bit.
 *
 * This allows for efficient encoding for passing across WASM.
 */
enum Button {
  A = 1,
  B = 1 << 1,
  Start = 1 << 2,
  Select = 1 << 3,
  Up = 1 << 4,
  Down = 1 << 5,
  Left = 1 << 6,
  Right = 1 << 7,
}

/**
 * Controls is used to control the state of the buttons, and to support remapping.
 */
class Controls {
  private mapping: Map<string, Button>;
  private state: number = 0;

  constructor() {
    this.mapping = new Map([
      ["w", Button.Up],
      ["a", Button.Left],
      ["s", Button.Down],
      ["d", Button.Right],
      ["g", Button.Select],
      ["h", Button.Start],
      ["j", Button.A],
      ["k", Button.B],
    ]);
  }

  /**
   * Update the state of an emulator with the current buttons being pressed.
   *
   * @param emu the emulator instance to update.
   */
  update(emu: Emulator) {
    emu.update_buttons(this.state);
  }

  /**
   * Update the button state when a key is pressed.
   *
   * @param key the string name for the key.
   */
  onKeyDown(key: string) {
    if (!this.mapping.has(key)) {
      return;
    }
    this.state |= this.mapping.get(key);
  }

  /**
   * Update the button state when a key is released.
   *
   * @param key the string name for the key.
   */
  onKeyUp(key: string) {
    if (!this.mapping.has(key)) {
      return;
    }
    this.state &= ~this.mapping.get(key);
  }
}

const audioCtx = new AudioContext();
const SAMPLE_RATE = 44100;

const emu = new Emulator(SAMPLE_RATE, audioCtx);

let settingsMenuOpen = false;
let paused = false;
let controls = new Controls();

function openSettingsMenu() {
  if (settingsMenuOpen) {
    return;
  }
  settingsMenuOpen = true;
  paused = true;
  const menu = document.getElementById("settings-menu-background");
  menu.classList.remove("invisible", "opacity-0");
  menu.classList.add("opacity-100");
}

function closeSettingsMenu() {
  if (!settingsMenuOpen) {
    return;
  }
  settingsMenuOpen = false;
  paused = false;
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

window.addEventListener("keydown", (ev) => {
  controls.onKeyDown(ev.key);
  controls.update(emu);
});
window.addEventListener("keyup", (ev) => {
  controls.onKeyUp(ev.key);
  controls.update(emu);
});

let old = 0.0;

function loop(timestamp: number) {
  const diff = 1000 * (timestamp - old);
  old = timestamp;
  if (!paused) {
    emu.step(ctx, diff);
  }

  window.requestAnimationFrame(loop);
}

window.requestAnimationFrame(loop);
