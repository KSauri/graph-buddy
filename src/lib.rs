extern crate num;

use num::NumCast;
use std::fmt;
use std::fs::File;
use std::num::ParseIntError;
use std::ops::Index;

extern crate image;
use image::{Rgb, RgbImage};

// TODO: start splitting this up into modules

// TODO: add better error messaging
// TODO: move this to a parsing module
#[derive(Debug)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error while parsing")
    }
}

impl From<std::io::Error> for ParseError {
    fn from(_e: std::io::Error) -> Self {
        Self {}
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_e: ParseIntError) -> Self {
        Self {}
    }
}

impl From<csv::Error> for ParseError {
    fn from(_e: csv::Error) -> Self {
        Self {}
    }
}

// TODO: add support for multiple columns
// TODO: revisit the structure of `data`
#[derive(Debug, Default, Clone)]
pub struct WorkSheet {
    data: Vec<(i64, i64)>,
}

impl Index<usize> for WorkSheet {
    type Output = (i64, i64);
    fn index(&self, idx: usize) -> &Self::Output {
        // TODO: this shouldn't reveal the underlying vec; maybe add a data type wrapper around the internal vectors?
        &self.data[idx]
    }
}

impl IntoIterator for WorkSheet {
    type Item = (i64, i64);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[derive(Default)]
pub struct WorkSheetBuilder<'a> {
    csv_data: Option<&'a str>,
    vec_data: Option<Vec<i64>>,
}

impl<'a> WorkSheetBuilder<'a> {
    pub fn new() -> Self {
        Self {
            csv_data: None,
            vec_data: None,
        }
    }

    pub fn csv_data(&mut self, csv_data: &'a str) -> Self {
        let mut result: WorkSheetBuilder = Default::default();
        result.csv_data = Some(csv_data);
        result
    }

    pub fn vec_data<T: NumCast + Clone>(&mut self, vec_data: Vec<T>) -> Self {
        let mut result: WorkSheetBuilder = Default::default();
        result.vec_data = vec_data.iter().map(|v| NumCast::from(v.clone())).collect();
        result
    }

    pub fn build(self) -> Result<WorkSheet, ParseError> {
        if self.csv_data.is_some() {
            if self.csv_data.is_none() {
                return Err(ParseError);
            }
            // TODO: move CSV package behind an API (facade)
            // TODO: this should be moved to `csv_data()` - or better yet dispatch to a private `csv_build` method and just match here?
            let file = File::open(self.csv_data.unwrap())?; // TODO: better handling on this unwrap
            let mut rdr = csv::Reader::from_reader(file);
            let mut data: Vec<(i64, i64)> = vec![];
            for result in rdr.records() {
                let record = result?;
                data.push((
                    record.get(0).ok_or_else(|| ParseError)?.parse::<i64>()?,
                    record.get(1).ok_or_else(|| ParseError)?.parse::<i64>()?,
                ));
            }
            Ok(WorkSheet { data })
        } else {
            Ok(WorkSheet { data: vec![] })
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct Point {
    x: i64,
    y: i64,
}

impl Point {
    pub fn transform(self, height: i64) -> Point {
        Point {
            x: self.x,
            y: height - self.y,
        }
    }

    pub fn scaled(&self, length: i64) -> Vec<Point> {
        if length == 0 {
            return vec![self.clone()];
        }

        let mut result = vec![];
        let (start_x, end_x) = (self.x - length, self.x + length);
        let (start_y, end_y) = (self.y - length, self.y + length);
        for current_x in start_x..=end_x {
            for current_y in start_y..=end_y {
                result.push(Point {
                    x: current_x,
                    y: current_y,
                });
            }
        }
        result
    }
}

pub trait Drawable {
    fn draw(&mut self);
}

pub struct Canvas {
    filename: &'static str,
    border: i64, // TODO: fix this usize vs i64 etc madness
    image_buffer: RgbImage,
    height: i64,
    worksheet: WorkSheet,
}

enum Axis {
    X,
    Y,
}

impl Canvas {
    pub fn new(border: i64, filename: &'static str, height: i64, worksheet: WorkSheet) -> Self {
        Self {
            filename,
            border,
            image_buffer: RgbImage::new(height as u32, height as u32),
            height,
            worksheet,
        }
    }

    fn save(&mut self) {
        self.image_buffer.save(self.filename).unwrap();
    }

    fn fill_in_white(&mut self) {
        for (_, _, pixel) in self.image_buffer.enumerate_pixels_mut() {
            *pixel = image::Rgb([255, 255, 255]);
        }
    }

    fn draw_tick(&mut self, point: Point, axis: Axis) {
        match axis {
            Axis::Y => {
                self.image_buffer
                    .put_pixel((point.x + 1) as u32, point.y as u32, Rgb([255, 0, 0]));
                self.image_buffer
                    .put_pixel((point.x + 2) as u32, point.y as u32, Rgb([255, 0, 0]));
            }
            Axis::X => {
                self.image_buffer
                    .put_pixel(point.x as u32, (point.y - 1) as u32, Rgb([255, 0, 0]));
                self.image_buffer
                    .put_pixel(point.x as u32, (point.y - 2) as u32, Rgb([255, 0, 0]));
            }
        }
    }

    fn draw_axis(&mut self) {
        for y in 0..self.height {
            if y % 10 == 0 {
                self.draw_tick(Point { x: self.border, y }, Axis::Y);
            }
            self.image_buffer
                .put_pixel(self.border as u32, y as u32, Rgb([255, 0, 0]));
        }

        // draw bottom axis
        for x in 0..self.height {
            if x % 10 == 0 {
                self.draw_tick(
                    Point {
                        x: x,
                        y: self.height - self.border,
                    },
                    Axis::X,
                );
            }
            self.image_buffer.put_pixel(
                x as u32,
                (self.height - self.border) as u32,
                Rgb([255, 0, 0]),
            );
        }
    }

    fn draw_point(&mut self, mut point: Point) {
        point = point.transform(self.height);
        let scaled_points = point.scaled(2);
        for scaled_point in scaled_points.iter() {
            self.image_buffer.put_pixel(
                (scaled_point.x + self.border) as u32,
                (scaled_point.y - self.border) as u32,
                Rgb([200, 0, 0]),
            );
        }
    }

    fn draw_points(&mut self) {
        self.worksheet
            .clone()
            .into_iter()
            .for_each(|(x, y)| self.draw_point(Point { x, y }));
    }
}

impl Drawable for Canvas {
    fn draw(&mut self) {
        self.fill_in_white();
        self.draw_axis();
        self.draw_points();
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worksheet_from_csv() {
        let ws: WorkSheet = WorkSheetBuilder::new()
            .csv_data("data/worksheet1.csv")
            .build()
            .unwrap();

        assert_eq!(ws[0].0, 1);
        assert_eq!(ws[1].1, 4);
    }

    #[test]
    fn incorrectly_formatted_worksheet_from_csv() {
        let ws = WorkSheetBuilder::new()
            .csv_data("data/improper1.csv")
            .build();

        match ws {
            Ok(_) => panic!("unexpected"),
            Err(ParseError) => {}
        }
    }

    #[test]
    fn transform_point() {
        let inputs = vec![
            Point { x: 1, y: 10 },
            Point { x: 5, y: 20 },
            Point { x: 17, y: 23 },
        ];

        let height = 100;
        let expected = vec![
            Point { x: 1, y: 90 },
            Point { x: 5, y: 80 },
            Point { x: 17, y: 77 },
        ];

        for (idx, &input) in inputs.iter().enumerate() {
            assert_eq!(input.transform(height), expected[idx]);
        }
    }

    #[test]
    fn get_scaled_point() {
        let point = Point { x: 10, y: 20 };
        let mut expected = vec![
            Point { x: 8, y: 18 },
            Point { x: 9, y: 18 },
            Point { x: 10, y: 18 },
            Point { x: 11, y: 18 },
            Point { x: 12, y: 18 },
            Point { x: 8, y: 19 },
            Point { x: 9, y: 19 },
            Point { x: 10, y: 19 },
            Point { x: 11, y: 19 },
            Point { x: 12, y: 19 },
            Point { x: 8, y: 20 },
            Point { x: 9, y: 20 },
            Point { x: 10, y: 20 },
            Point { x: 11, y: 20 },
            Point { x: 12, y: 20 },
            Point { x: 8, y: 21 },
            Point { x: 9, y: 21 },
            Point { x: 10, y: 21 },
            Point { x: 11, y: 21 },
            Point { x: 12, y: 21 },
            Point { x: 8, y: 22 },
            Point { x: 9, y: 22 },
            Point { x: 10, y: 22 },
            Point { x: 11, y: 22 },
            Point { x: 12, y: 22 },
        ];
        let mut actual = point.scaled(2);
        expected.sort();
        actual.sort();

        assert_eq!(actual, expected);
    }

    // TODO: add test for vectors
    // TODO: add more tests for incorrectly formatted CSVs
    // TODO: figure out how to test side effects (aka a png being created)
}
