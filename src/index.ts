import { Console } from "../Cargo.toml";


const emu = new Console(600, 600);

const romSelector = document.getElementById("rom-selector") as HTMLInputElement;
romSelector.addEventListener("change", function () {
    this.files.item(0).arrayBuffer().then(rom => {
        console.log(new Uint8Array(rom));
    });
}, false);

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");

let old = 0.0;

function loop(timestamp: number) {
    const diff = 1000 * (timestamp - old)
    old = timestamp;
    emu.draw(ctx, diff)

    window.requestAnimationFrame(loop);
}

window.requestAnimationFrame(loop);