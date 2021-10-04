extern crate image;
use graph_buddy::{Drawable, PngCanvas};

fn main() {
    let mut canvas = PngCanvas::new(10, "foo.png");
    canvas.draw();
}
