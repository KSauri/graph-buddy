extern crate num;

use num::NumCast;
use std::fmt;
use std::fs::File;
use std::num::ParseIntError;
use std::ops::Index;

pub fn main() {}

#[derive(Debug, Clone)]
pub struct ParseError;

type Result<T> = std::result::Result<T, ParseError>;

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

#[derive(Debug, Default)]
pub struct WorkSheet {
    data: Vec<(i64, i64)>,
}

impl WorkSheet {}

impl Index<usize> for WorkSheet {
    type Output = (i64, i64);
    fn index(&self, idx: usize) -> &Self::Output {
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

    pub fn build(self) -> Result<WorkSheet> {
        let data = if self.csv_data.is_some() {
            let file = File::open(self.csv_data.unwrap())?;
            let mut rdr = csv::Reader::from_reader(file);
            let mut data: Vec<(i64, i64)> = vec![];
            for result in rdr.records() {
                let record = result.unwrap();
                data.push((
                    record.get(0).ok_or_else(|| ParseError)?.parse::<i64>()?,
                    record.get(1).ok_or_else(|| ParseError)?.parse::<i64>()?,
                ));
            }
            data
        } else {
            vec![]
        };
        Ok(WorkSheet { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worksheet_from_csv() {
        let ws: WorkSheet = WorkSheetBuilder::new()
            .csv_data("data/WorkSheet1.csv")
            .build()
            .unwrap();

        assert_eq!(ws[0].0, 1);
        assert_eq!(ws[1].1, 4);
    }
}
