use crate::consts::{level_to_severity, Facility};
use log::Record;
use log4rs::encode::writer::simple::SimpleWriter;
use log4rs::encode::Encode;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Format(pub Arc<dyn Encode>);

impl Default for Format {
    fn default() -> Self {
        Format(Arc::new(log4rs::encode::pattern::PatternEncoder::default()))
    }
}

impl Format {
    pub fn encoder<E: Encode>(mut self, encoder: E) -> Self {
        self.0 = Arc::new(encoder) as Arc<dyn Encode + 'static>;
        self
    }
}

impl log4rs::encode::Encode for Format {
    fn encode(
        &self,
        w: &mut dyn log4rs::encode::Write,
        record: &Record<'_>,
    ) -> Result<(), anyhow::Error> {
        let mut buf: Vec<u8> = Vec::new();
        self.0.encode(&mut SimpleWriter(&mut buf), record)?;
        let msg = String::from_utf8_lossy(&buf);

        let priority = Facility::USER as u8 | level_to_severity(record.level());
        let msg = format!("<{}> {}\n", priority, msg);
        w.write_all(msg.as_bytes())?;
        Ok(())
    }
}
