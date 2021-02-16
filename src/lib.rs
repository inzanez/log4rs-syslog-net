use log::Record;
use log4rs::append::Append;
use log4rs::encode::Encode;
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{sync_channel, SyncSender};

pub mod consts;
pub mod plain;
pub mod rfc5424;
pub mod rfc5425;

const DEFAULT_PORT: u16 = 514;
const DEFAULT_ADDRESS: &str = "localhost:514";

/// Syslog message format.
#[derive(Debug)]
pub enum MessageFormat {
    /// No formatting is applied.
    Plain(plain::Format),
    /// Formatting according to RFC5424 is applied.
    Rfc5424(rfc5424::Format),
    /// Formatting according to RFC5425 is applied (use with telegraf).
    Rfc5425(rfc5425::Format),
}

impl MessageFormat {
    fn format(
        &self,
        w: &mut dyn std::io::Write,
        rec: &Record<'_>,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut w = log4rs::encode::writer::simple::SimpleWriter(w);
        match self {
            MessageFormat::Plain(fmt) => fmt.encode(&mut w, &rec),
            MessageFormat::Rfc5424(fmt) => fmt.encode(&mut w, &rec),
            MessageFormat::Rfc5425(fmt) => fmt.encode(&mut w, &rec),
        }
    }
}

/// Appender that sends log messages to syslog.
#[derive(Debug)]
pub struct SyslogAppender {
    addr: String,
    writer: SyncSender<Vec<u8>>,
    msg_format: MessageFormat,
}

impl<'a> Append for SyslogAppender {
    fn append(&self, record: &Record<'_>) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut v = vec![];
        // Format the message, which will be different based on the chosen MsgFormat
        self.msg_format.format(&mut v, &record)?;

        self.writer.send(v)?;

        Ok(())
    }

    fn flush(&self) {}
}

/// Builder for `SyslogAppender`.
pub struct SyslogAppenderBuilder {
    addrs: String,
    msg_format: MessageFormat,
}

impl SyslogAppenderBuilder {
    /// Creates a `SyslogAppenderBuilder` for constructing new `SyslogAppender`.
    pub fn new() -> SyslogAppenderBuilder {
        SyslogAppenderBuilder {
            addrs: DEFAULT_ADDRESS.to_string(),
            msg_format: MessageFormat::Plain(plain::Format(Box::new(
                log4rs::encode::pattern::PatternEncoder::default(),
            ))),
        }
    }

    /// Sets network address of syslog server.
    ///
    /// Defaults to "localhost:514".
    pub fn address(mut self, addrs: String) -> Self {
        self.addrs = addrs;
        self
    }

    /// Sets type of log message formatter.
    ///
    /// Defaults to `Plain`.
    pub fn format(mut self, mf: MessageFormat) -> Self {
        self.msg_format = mf;
        self
    }

    /// Produces a `SyslogAppender` with parameters, supplied to the builder.
    pub fn build(mut self) -> std::io::Result<SyslogAppender> {
        // norm_addrs(&mut self.addrs);
        if self.addrs.find(':').is_none() {
            self.addrs.push(':');
            self.addrs.push_str(&DEFAULT_PORT.to_string())
        }
        let (tx, rx) = sync_channel(12);
        let mut conn = TcpStream::connect(&self.addrs)?;

        std::thread::spawn(move ||{
            loop {
                let v: Vec<u8> = rx.recv().unwrap();
                if let Err(e) = conn.write(&v){
                    drop(e);
                };
            }
        });

        let appender = SyslogAppender {
            addr: self.addrs,
            writer: tx,
            msg_format: self.msg_format,
        };

        Ok(appender)
    }
}
