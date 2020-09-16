use crate::consts::{level_to_severity, Facility};
use crate::{Formattable, SyslogAppenderProtocol};
use log::Record;
use log4rs::encode::writer::simple::SimpleWriter;
use std::error::Error;

#[derive(Debug)]
pub struct Format {}

impl Formattable for Format {
    fn format<'a>(
        &self,
        record: &Record<'a>,
        _protocol: &SyslogAppenderProtocol,
        encoder: &Box<dyn log4rs::encode::Encode>,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        let mut buf: Vec<u8> = Vec::new();
        encoder.encode(&mut SimpleWriter(&mut buf), record)?;
        let msg = std::str::from_utf8(&buf).unwrap();

        let priority = Facility::USER as u8 | level_to_severity(record.level());
        let msg = format!("<{}> {}\n", priority, msg);

        Ok(msg)
    }
}
