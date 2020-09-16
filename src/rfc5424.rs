use crate::consts::{level_to_severity, Facility, NILVALUE};
use crate::{Formattable, SyslogAppenderProtocol};
use log::Record;
use log4rs::encode::writer::simple::SimpleWriter;
use std::error::Error;

#[derive(Debug)]
pub struct Format {
    facility: Facility,
    hostname: String,
    app_name: String,
    proc_id: String,
    bom: bool,
}

impl Format {
    pub fn new() -> Self {
        Format {
            facility: Facility::LOCAL0,
            hostname: "".to_string(),
            app_name: "".to_string(),
            proc_id: format!("{}", std::process::id()),
            bom: false,
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

    pub fn app_name<S: Into<String>>(mut self, app_name: S) -> Self {
        self.app_name = app_name.into();
        self
    }

    pub fn proc_id<S: Into<String>>(mut self, proc_id: S) -> Self {
        self.proc_id = proc_id.into();
        self
    }

    pub fn bom(mut self, bom: bool) -> Self {
        self.bom = bom;
        self
    }
}
impl Formattable for Format {
    fn format<'a>(
        &self,
        record: &Record<'a>,
        _protocol: &SyslogAppenderProtocol,
        encoder: &Box<dyn log4rs::encode::Encode>,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        let priority = self.facility as u8 | level_to_severity(record.level());
        let msg_id = 0;
        let struct_data = NILVALUE;
        let bom_str;
        if self.bom {
            bom_str = "\u{EF}\u{BB}\u{BF}";
        } else {
            bom_str = "";
        }

        let mut buf: Vec<u8> = Vec::new();
        encoder.encode(&mut SimpleWriter(&mut buf), record)?;
        let msg = std::str::from_utf8(&buf).unwrap();

        let msg = format!(
            "<{}>{} {} {} {} {} {} {} {}{}\n",
            priority,
            1,
            chrono::Utc::now(),
            self.hostname,
            self.app_name,
            self.proc_id,
            msg_id,
            struct_data,
            bom_str,
            msg
        );

        Ok(msg)
    }
}
