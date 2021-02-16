use crate::consts::{level_to_severity, Facility};
use log::Record;
use log4rs::encode::writer::simple::SimpleWriter;
use std::error::Error;

#[derive(Debug)]
pub struct Format(pub Box<dyn log4rs::encode::Encode>);

impl log4rs::encode::Encode for Format {
    fn encode(
        &self,
        w: &mut dyn log4rs::encode::Write,
        record: &Record<'_>,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut buf: Vec<u8> = Vec::new();
        self.0.encode(&mut SimpleWriter(&mut buf), record)?;
        let msg = String::from_utf8_lossy(&buf);

        let priority = Facility::USER as u8 | level_to_severity(record.level());
        let msg = format!("<{}> {}\n", priority, msg);
        w.write_all(msg.as_bytes())?;
        Ok(())
    }
}
