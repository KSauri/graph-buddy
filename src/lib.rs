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
#[derive(Debug, Clone)]
pub struct ParseError;

type WSResult<T> = std::result::Result<T, ParseError>;

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
#[derive(Debug, Default)]
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

    fn build_from_csv(&self) -> WSResult<WorkSheet> {
        if self.csv_data.is_none() {
            return Err(ParseError {});
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

    pub fn build(self) -> WSResult<WorkSheet> {
        if self.csv_data.is_some() {
            self.build_from_csv()
        } else {
            Ok(WorkSheet { data: vec![] })
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn transform(self, height: u32) -> Point {
        Point {
            x: self.x,
            y: height - self.y,
        }
    }
}

pub trait Drawable {
    fn draw(&mut self);
}

pub struct PngCanvas {
    filename: &'static str,
    border: u32,
    image_buffer: RgbImage,
}

impl PngCanvas {
    pub fn new(border: u32, filename: &'static str) -> Self {
        Self {
            filename,
            border,
            image_buffer: RgbImage::new(600, 600),
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

    fn draw_axis(&mut self) {
        for y in 0..600 {
            if y % 10 == 0 {
                self.image_buffer
                    .put_pixel(self.border + 1, y, Rgb([255, 0, 0]));
                self.image_buffer
                    .put_pixel(self.border + 2, y, Rgb([255, 0, 0]));
            }
            self.image_buffer
                .put_pixel(self.border, y, Rgb([255, 0, 0]));
        }

        // draw bottom axis
        for x in 0..600 {
            if x % 10 == 0 {
                self.image_buffer
                    .put_pixel(x, 600 - self.border - 1, Rgb([255, 0, 0]));
                self.image_buffer
                    .put_pixel(x, 600 - self.border - 2, Rgb([255, 0, 0]));
            }
            self.image_buffer
                .put_pixel(x, 600 - self.border, Rgb([255, 0, 0]));
        }
    }
}

impl Drawable for PngCanvas {
    fn draw(&mut self) {
        self.fill_in_white();
        self.draw_axis();
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
            Err(ParseError {}) => {}
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

    // TODO: add test for vectors
    // TODO: add more tests for incorrectly formatted CSVs
    // TODO: figure out how to test side effects (aka a png being created)
}
