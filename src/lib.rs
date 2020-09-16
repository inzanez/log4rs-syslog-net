use std::net::{UdpSocket, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Mutex;
use log4rs::append::Append;
use std::error::Error;
use log::Record;
use std::io;
use std::io::Write;

pub mod consts;
pub mod plain;
pub mod rfc5424;
pub mod rfc5425;

const DEFAULT_PROTOCOL: SyslogAppenderProtocol = SyslogAppenderProtocol::UDP;
const DEFAULT_PORT: u16 = 514;
const DEFAULT_ADDRESS: &'static str = "localhost:514";
const DEFAULT_MAX_LENGTH: usize = 2048;

/// Writers to send syslog to UDP or TCP
#[derive(Debug)]
enum SyslogWriter {
	Udp(UdpSocket, SocketAddr),
	Tcp(Mutex<TcpStream>)
}

/// Trait that allows to format a given message.
trait Formattable {
	fn format(&self, rec: &Record, protocol: &SyslogAppenderProtocol) -> String;
}

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

impl Formattable for MessageFormat {
	fn format<'a>(&self, rec: &Record<'a>, protocol: &SyslogAppenderProtocol) -> String {
		match self {
			MessageFormat::Plain(ref fmt) => fmt.format(&rec, protocol),
			MessageFormat::Rfc5424(ref fmt) => fmt.format(&rec, protocol),
			MessageFormat::Rfc5425(ref fmt) => fmt.format(&rec, protocol),
		}
	}
}

#[derive(Debug)]
pub enum SyslogAppenderProtocol {
	UDP,
	TCP,
}

/// Appender that sends log messages to syslog.
#[derive(Debug)]
pub struct SyslogAppender {
	writer: SyslogWriter,
	msg_format: MessageFormat,
	max_len: usize,
	protocol: SyslogAppenderProtocol,
}

impl Append for SyslogAppender {
	fn append<'a>(&self, record: &Record<'a>) -> Result<(), Box<dyn Error + Sync + Send>> {
		// Format the message, which will be different based on the chosen MsgFormat
		let msg = self.msg_format.format(&record, &self.protocol);

		let mut bytes = msg.as_bytes();

		// Check for message length. If it exceeds DEFAULT_MAX_LENGTH, truncate.
		// Per syslog specification, a receiver may discard messages exceeding that limit.
		if bytes.len() > DEFAULT_MAX_LENGTH {
			bytes = &bytes[0..DEFAULT_MAX_LENGTH];
		}

		// Write to UDP or TCP depending on the configuration
		match self.writer {
			SyslogWriter::Udp(ref socket, ref addrs) => {
				socket.send_to(&bytes, addrs)?;
			},
			SyslogWriter::Tcp(ref stream_w) => {
				let mut stream = stream_w.lock().unwrap();
				stream.write(bytes)?;
			}
		};
		Ok(())
	}

	fn flush(&self) {}
}

/// Builder for `SyslogAppender`.
pub struct SyslogAppenderBuilder {
	protocol: SyslogAppenderProtocol,
	addrs: String,
	max_len: usize,
	msg_format: MessageFormat
}

impl SyslogAppenderBuilder {
	/// Creates a `SyslogAppenderBuilder` for constructing new `SyslogAppender`.
	pub fn new() -> SyslogAppenderBuilder {
		SyslogAppenderBuilder {
			protocol: DEFAULT_PROTOCOL,
			addrs: DEFAULT_ADDRESS.to_string(),
			max_len: DEFAULT_MAX_LENGTH,
			msg_format: MessageFormat::Plain(plain::Format{}),
		}
	}

	/// Sets network protocol for accessing syslog.
	///
	/// Defaults to "udp".
	pub fn protocol(&mut self, p: SyslogAppenderProtocol) -> &mut SyslogAppenderBuilder {
		self.protocol = p;
		self
	}

	/// Sets network address of syslog server.
	///
	/// Defaults to "localhost:514".
	pub fn address(&mut self, addrs: String) -> &mut SyslogAppenderBuilder {
		self.addrs = addrs;
		self
	}

	/// Sets type of log message formatter.
	///
	/// Defaults to `Plain`.
	pub fn format(&mut self, mf: MessageFormat) -> &mut SyslogAppenderBuilder {
		self.msg_format = mf;
		self
	}

    /// Sets the maximum length of a message in bytes. If a log message exceedes
    /// this size, it's truncated with not respect to UTF char boundaries.
    ///
    /// Defaults to 2048.
    pub fn max_len(&mut self, ml: usize) -> &mut SyslogAppenderBuilder {
		self.max_len = ml;
		self
	}

	/// Produces a `SyslogAppender` with parameters, supplied to the builder.
	pub fn finalize(mut self) -> io::Result<SyslogAppender> {
		norm_addrs(&mut self.addrs);
		let writer;

		match self.protocol {
			SyslogAppenderProtocol::UDP => {
				writer = udp_writer(self.addrs.as_str());
			},
			SyslogAppenderProtocol::TCP => {
				writer = tcp_writer(self.addrs.as_str());
			},
		}

		let appender = SyslogAppender {
			writer,
			msg_format: self.msg_format,
			max_len: self.max_len,
			protocol: self.protocol,
		};

		Ok(appender)
	}
}

/// Normalizes network address -- adds port if necessary
fn norm_addrs(addrs: &mut String) {
	if !addrs.find(':').is_some() {
		addrs.push(':');
		addrs.push_str(&DEFAULT_PORT.to_string())
	}
}

/// Creates writer for UDP protocol based on external host and port
fn udp_writer<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
	let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
	let rem_addrs = rem.to_socket_addrs().unwrap().next().unwrap();
	SyslogWriter::Udp(socket, rem_addrs)
}

/// Creates writer for TCP protocol based on external host and port
fn tcp_writer<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
	let stream = TcpStream::connect(rem).unwrap();
	SyslogWriter::Tcp(Mutex::new(stream))
}
