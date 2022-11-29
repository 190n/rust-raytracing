rust-raytracing
===============

My implementation of [_Ray Tracing in One Weekend_](https://raytracing.github.io/books/RayTracingInOneWeekend.html) and (part of) [_Ray Tracing: The Next Week_](https://raytracing.github.io/books/RayTracingTheNextWeek.html) in Rust.

## Progress

- [x] basic RTWeekend features
- [x] multithreading
- [x] argument parsing
- [x] BVH
- [ ] motion blur
- [ ] textures (solid, noise, images)
- [ ] rectangles and lights
- [ ] instances
- [ ] volumes
- [ ] final scene from _The Next Week_
- [ ] PNG output?
- [ ] mesh geometry?

## Usage

With Rust and Cargo installed, you can use `cargo run --release` to run with the default settings (including output to stdout), or `cargo run --release -- <ARGS>` to run with different arguments. Or after compiling it (one of the `run` commands or `cargo build --release`), you can run the executable in `./target/release` directly without using `cargo`.

```
usage: raytracing [-t|--threads n] [-w|--width w] [-s|--samples s] [-r|--seed r] 
         [-d|--depth d] [-o|--output filename] [-S|--scene scene]

  -t, --threads n:       number of threads. default: number of logical processors
  -w, --width w:         width of image in pixels. default: 600
  -s, --samples s:       number of samples per pixel. default: 100
  -d, --depth d:         maximum bounces per ray. default: 50
  -r, --seed r:          random number seed. default: entropy from the OS
  -o, --output filename: file to output PPM image to. default: stdout
  -S, --scene scene:     which scene to render. options:
    random:
      random spheres; final render from Ray Tracing in One Weekend
    figure19:
      figure 19 from Ray Tracing in One Weekend; three spheres with different materials
    refraction:
      a series of spheres lowering into a refractive material
    default: random
```
