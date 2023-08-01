use crate::consts::{level_to_severity, Facility, NILVALUE};
use log::Record;
use log4rs::encode::writer::simple::SimpleWriter;
use log4rs::encode::Encode;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Format {
    facility: Facility,
    hostname: String,
    app_name: String,
    proc_id: String,
    encoder: std::sync::Arc<dyn log4rs::encode::Encode>,
}

impl Default for Format {
    fn default() -> Self {
        Format::new()
    }
}

impl Format {
    pub fn new() -> Self {
        Format {
            facility: Facility::LOCAL0,
            hostname: "".to_string(),
            app_name: "".to_string(),
            proc_id: format!("{}", std::process::id()),
            encoder: Arc::new(log4rs::encode::pattern::PatternEncoder::default()),
        }
    }

    pub fn facility(mut self, facility: Facility) -> Self {
        self.facility = facility;
        self
    }

    pub fn hostname<S: Into<String>>(mut self, hostname: S) -> Self {
        self.hostname = hostname.into();
        self
    }
    pub fn encoder<E: Encode>(mut self, encoder: E) -> Self {
        self.encoder = Arc::new(encoder) as Arc<dyn Encode + 'static>;
        self
    }

    pub fn app_name<S: Into<String>>(mut self, app_name: S) -> Self {
        self.app_name = app_name.into();
        self
    }

    pub fn proc_id<S: Into<String>>(mut self, proc_id: S) -> Self {
        self.proc_id = proc_id.into();
        self
    }
}

impl log4rs::encode::Encode for Format {
    fn encode(
        &self,
        w: &mut dyn log4rs::encode::Write,
        record: &Record<'_>,
    ) -> Result<(), anyhow::Error> {
        let priority = self.facility as u8 | level_to_severity(record.level());
        let msg_id = 0;
        let struct_data = NILVALUE;

        let mut buf: Vec<u8> = Vec::new();
        self.encoder.encode(&mut SimpleWriter(&mut buf), record)?;
        let msg = String::from_utf8_lossy(&buf);

        let msg = format!(
            "<{}>{} {} {} {} {} {} {} {}\n",
            priority,
            1,
            chrono::Utc::now(),
            self.hostname,
            self.app_name,
            self.proc_id,
            msg_id,
            struct_data,
            msg
        );

        w.write_all(msg.as_bytes())?;

        Ok(())
    }
}
