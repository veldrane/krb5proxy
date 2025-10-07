// Removed conflicting import: use tokio::net::unix::pipe::Sender;
use tokio::sync::mpsc;
use chrono::prelude::*;

pub struct Logger {
    tx: mpsc::Sender<LogMessage>,
    _logger: tokio::task::JoinHandle<()>,
}

enum LogType {
    Info,
    Warning,
    Error,
}

struct LogMessage(LogType,String);

impl Logger {

    pub fn build(output: &str) -> Self {
        
        let log_output = get_output(output);
        let (tx, rx): (mpsc::Sender<LogMessage>, mpsc::Receiver<LogMessage>) = mpsc::channel(100);
        let logger_task =   tokio::spawn(log_receiver(rx, log_output));
        let logger = Logger {
            tx,
            _logger: logger_task,
        };

        let _ = logger.tx.try_send(LogMessage(LogType::Info, "Starting logger".into()));
        logger
    }

    pub async fn info(&self, message: String) {
        let _ = self.tx.send(LogMessage(LogType::Info, message)).await;
    }

    pub async fn warn(&self, message: String) {
        let _ = self.tx.send(LogMessage(LogType::Warning, message)).await;
    }

    pub async fn error(&self, message: String) {
        let _ = self.tx.send(LogMessage(LogType::Error, message)).await;
    }
}




async fn log_receiver<T: LogOutput + Send + 'static>(mut rx: mpsc::Receiver<LogMessage>, logger: T) {
    while let Some(log_message) = rx.recv().await {
        logger.log(log_message);
    }
}


fn get_output(output: &str) -> impl LogOutput + Send + 'static {
    match output {
        "console" => ConsoleLogger,
        _ => ConsoleLogger,
    }
}

trait LogOutput {
    fn log(&self, log_message: LogMessage);
}

struct ConsoleLogger;

impl LogOutput for ConsoleLogger {
    fn log(&self, log_message: LogMessage) {
        let timestamp =  Utc::now().to_string();
        match log_message.0 {
            LogType::Info => println!("{} {} {}",timestamp,"---| Info:", log_message.1),
            LogType::Warning => println!("{}  ---| Warning: {}",timestamp, log_message.1),
            LogType::Error => println!("{} {} {}",timestamp,"---| Error:", log_message.1),
        }
    }
}