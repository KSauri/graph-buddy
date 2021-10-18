extern crate image;
use graph_buddy::{Canvas, Drawable, WorkSheet, WorkSheetBuilder};

fn main() {
    let ws: WorkSheet = WorkSheetBuilder::new()
        .csv_data("data/worksheet1.csv")
        .build()
        .unwrap();
    let mut canvas = Canvas::new(10, "foo.png", 600, ws);
    canvas.draw();
}
