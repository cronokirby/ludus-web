import { draw } from "../Cargo.toml";

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");

draw(ctx, 600, 600);
