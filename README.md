# ludus-web

I wrote an NES emulator, [ludus](https://github.com/cronokirby/ludus), a while back, in Rust.
I spent a bit of time porting it to WASM, so that it could run in a browser, and this was the result.

The project is deployed at [ludus-web.cronokirby.com](https://ludus-web.cronokirby.com) if you'd
like to try it out.

## Development

First, you'll need to install Rust, and its related tooling. Then, you'll need
to install [wasm-pack](https://github.com/rustwasm/wasm-pack) to build the Rust bits.
From there, running `npm install` should work to get all of the dependencies.

To run the project in development:

```
npm run dev
```

To build the project for production:

```
npm run build
```

which will create the output files in the `dist` directory.
