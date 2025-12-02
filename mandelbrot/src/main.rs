pub mod library;

fn main() {
    library::function();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT", args[0]);
        eprintln!(
            "example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        );
        std::process::exit(1);
    }

    let filename = &args[1];
    let bounds = library::common::environment::parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions");
    let upper_left = library::common::environment::parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let lower_right = library::common::environment::parse_complex(&args[4])
        .expect("error parsing lower right corner point");
    let mut pixels = vec![0; bounds.0 * bounds.1];

    let threads = num_cpus::get();
    println!("Number of CPU cors: {}", threads);

    let rows_per_band = bounds.1 / threads + 1;
    let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            let top = rows_per_band * i;
            let height = band.len() / bounds.0;
            let band_bounds = (bounds.0, height);
            let band_upper_left = library::math::mandelbrot::pixel_to_point(
                bounds,
                (0, top),
                upper_left,
                lower_right,
            );
            let band_lower_right = library::math::mandelbrot::pixel_to_point(
                bounds,
                (bounds.0, top + height),
                upper_left,
                lower_right,
            );
            spawner.spawn(move |_| {
                library::math::mandelbrot::render(
                    band,
                    band_bounds,
                    band_upper_left,
                    band_lower_right,
                );
            });
        }
    })
    .unwrap();
    // library::math::mandelbrot::render(&mut pixels, bounds, upper_left, lower_right);

    library::math::mandelbrot::write_image(filename, &pixels, bounds)
        .expect("error writing PNG file");
}
