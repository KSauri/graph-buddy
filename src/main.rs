extern crate image;
use graph_buddy::{Canvas, Drawable};

fn main() {
    let mut canvas = Canvas::new(10, "foo.png");
    canvas.draw();
}
