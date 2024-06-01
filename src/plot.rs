use plotters::prelude::*;

pub fn plot_graph(xs: &[f32], ys: &[f32], filename: &str) {
    let filepath = format!("plots/{}", filename);
    let root = BitMapBackend::new(&filepath, (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let x_min = xs.iter().cloned().fold(f32::MAX, f32::min);
    let x_max = xs.iter().cloned().fold(f32::MIN, f32::max);
    let y_min = ys.iter().cloned().fold(f32::MAX, f32::min);
    let y_max = ys.iter().cloned().fold(f32::MIN, f32::max);

    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            xs.iter().zip(ys.iter()).map(|(&x, &y)| (x, y)),
            &BLUE,
        ))
        .unwrap();
}
