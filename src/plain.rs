use crate::Formattable;
use log::Record;
use crate::consts::{Facility, level_to_severity};

#[derive(Debug)]
pub struct Format {}

impl Formattable for Format {
    fn format<'a>(&self, record: &Record<'a>) -> String {
        let priority = Facility::USER as u8 | level_to_severity(record.level());
        let msg = format!("<{}> {}\n",
                          priority,
                          record.args()
        );

        msg
    }
}