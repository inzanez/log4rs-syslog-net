use crate::{Formattable, SyslogAppenderProtocol};
use log::Record;
use crate::consts::{Facility, level_to_severity};

#[derive(Debug)]
pub struct Format {}

impl Formattable for Format {
    fn format<'a>(&self, record: &Record<'a>, _protocol: &SyslogAppenderProtocol) -> String {
        let priority = Facility::USER as u8 | level_to_severity(record.level());
        let msg = format!("<{}> {}\n",
                          priority,
                          record.args()
        );

        msg
    }
}