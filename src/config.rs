#![forbid(unsafe_code)]

use serde::Deserialize;
use crate::consts::Facility;
use crate::MessageFormat;
use crate::SyslogAppenderBuilder;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageFormatKind {
    Plain,
    Rfc5424,
    Rfc5425,
}

impl Default for MessageFormatKind {
    fn default() -> MessageFormatKind {
        MessageFormatKind::Plain
    }
}

/// The configuration of syslog appender format.
#[derive(Deserialize)]
struct SyslogFormatConfig {
    #[serde(default)]
    kind: MessageFormatKind,
    facility: Option<Facility>,
    hostname: Option<String>,
    app_name: Option<String>,
    proc_id: Option<String>,
}

/// The configuration of syslog appender.
#[derive(Deserialize)]
struct SyslogAppenderConfig {
    address: Option<String>,
    format: Option<SyslogFormatConfig>,
    encoder: Option<log4rs::encode::EncoderConfig>,
}

struct SyslogAppenderDeserializer;

/// A deserializer for the `SyslogAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: syslog-net
/// 
/// # The values below are all optional.
/// # The listed value will be used when omitted.
///
/// # The address and port of the syslog server.
/// address: localhost:514
///
/// format:
///     # Can be one of plain, rfc5424 and rfc5425.
///     kind: plain
///     # rfc5424 and rfc5425 support the following additional options:
///     facility: LOCAL0
///     hostname: ""
///     app_name: ""
///     # Id of the current process
///     proc_id: 1111
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///     kind: pattern
/// ```
impl log4rs::config::Deserialize for SyslogAppenderDeserializer {
    type Trait = dyn log4rs::append::Append;
    type Config = SyslogAppenderConfig;

    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &log4rs::config::Deserializers,
    ) -> Result<Box<Self::Trait>, anyhow::Error> {
        let mut builder = SyslogAppenderBuilder::new();

        if let Some(address) = config.address {
            builder = builder.address(address);
        }

        let format_kind = config.format.as_ref().map(|ref fmt| fmt.kind.clone()).unwrap_or_default();

        match format_kind {
            MessageFormatKind::Plain => {
                let mut format = crate::plain::Format::default();

                if let Some(encoder_conf) = config.encoder {
                    format = format.encoder(deserializers.deserialize(&encoder_conf.kind, encoder_conf.config)?);
                };

                builder = builder.format(MessageFormat::Plain(format));
            }
            MessageFormatKind::Rfc5424 => {
                let mut format = crate::rfc5424::Format::new();

                if let Some(format_config) = config.format {
                    if let Some(facility) = format_config.facility {
                        format = format.facility(facility);
                    };
                    if let Some(hostname) = format_config.hostname {
                        format = format.hostname(hostname);
                    };
                    if let Some(app_name) = format_config.app_name {
                        format = format.app_name(app_name);
                    };
                    if let Some(proc_id) = format_config.proc_id {
                        format = format.proc_id(proc_id);
                    };
                }

                if let Some(encoder_conf) = config.encoder {
                    format = format.encoder(deserializers.deserialize(&encoder_conf.kind, encoder_conf.config)?);
                };

                builder = builder.format(MessageFormat::Rfc5424(format));
            }
            MessageFormatKind::Rfc5425 => {
                let mut format = crate::rfc5425::Format::new();

                if let Some(format_config) = config.format {
                    if let Some(facility) = format_config.facility {
                        format = format.facility(facility);
                    };
                    if let Some(hostname) = format_config.hostname {
                        format = format.hostname(hostname);
                    };
                    if let Some(app_name) = format_config.app_name {
                        format = format.app_name(app_name);
                    };
                    if let Some(proc_id) = format_config.proc_id {
                        format = format.proc_id(proc_id);
                    };
                }

                if let Some(encoder_conf) = config.encoder {
                    format = format.encoder(deserializers.deserialize(&encoder_conf.kind, encoder_conf.config)?);
                };

                builder = builder.format(MessageFormat::Rfc5425(format));
            }
        }

        let appender = builder.build();

        Ok(appender.map(Box::new)?)
    }
}

/// Register deserializer for creating syslog appender based on log4rs configuration file.
///
/// # Examples
///
/// ```rust
/// let mut deserializers = log4rs::config::Deserializers::new();
/// log4rs_syslog_net::register(&mut deserializers);
/// log4rs::init_file("/path/to/log-conf.yaml", deserializers).unwrap();
/// ```
pub fn register(deserializers: &mut log4rs::config::Deserializers) {
    deserializers.insert("syslog-net", SyslogAppenderDeserializer);
}
