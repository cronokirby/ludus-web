import { Console } from "../Cargo.toml";

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
const emu = new Console(600, 600);

let old = 0.0;

function loop(timestamp: number) {
    const diff = 1000 * (timestamp - old)
    old = timestamp;
    emu.draw(ctx, diff)

    window.requestAnimationFrame(loop);
}

window.requestAnimationFrame(loop);