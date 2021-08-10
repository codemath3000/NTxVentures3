#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate actix_web;

use slog::{Drain, Fuse, Level, Never, FilterFn, Record, LevelFilter, Logger, Discard};
use slog_async::Async;
use slog_term::{FullFormat, PlainDecorator, TermDecorator};
use std::fs::{OpenOptions, create_dir_all};
use std::iter::Filter;
use actix_web::{HttpRequest, HttpResponse};
use std::env::{vars, var};
use std::env::consts::OS;
use std::str::FromStr;
use std::io;
use std::io::prelude::*;

struct LoggingService {
    errorLogger: Logger,
    infoLogger: Logger,
    requestsLogger: Logger,
}

impl LoggingService {
    fn generateConsoleDrain() -> Fuse<Async> {
        return Async::new(FullFormat::new(TermDecorator::new().build()).build().fuse()).build().fuse();
    }

    fn generateFileDrain(directory: String, fileName: String) -> Fuse<Async> {
        let createDirResult = create_dir_all(std::path::Path::new(directory.as_str()));
        if createDirResult.is_err() {
            panic!("Log directory creation failed.");
        }
        let mut fullPath: String = directory.clone();
        fullPath.push_str(fileName.as_str());
        fullPath.push_str(".log");
        let fileResult = OpenOptions::new().create(true).write(true).truncate(true).open(fullPath);
        if fileResult.is_err() {
            panic!("Log file creation failed.");
        } else {
            let file = fileResult.unwrap();
            return Async::new(FullFormat::new(PlainDecorator::new(file)).build().fuse()).build().fuse();
        }
    }

    pub fn initStdout(useEnvVar: bool) -> Self {
        if useEnvVar {
            let mut logLevel = Level::Debug;
            let initialLogLevel = var("NTX_LOG_LEVEL");
            if initialLogLevel.is_err() {
                panic!("Could not obtain log level from environment variable.");
            } else {
                let processedLogLevel = Level::from_str(initialLogLevel.unwrap().as_str());
                if processedLogLevel.is_err() {
                    panic!("Invalid log level specified in environment variable.");
                } else {
                    logLevel = processedLogLevel.unwrap();
                }
            }
            return Self {
                errorLogger: Logger::root(Discard, o!()),
                infoLogger: Logger::root(Self::generateConsoleDrain().filter_level(logLevel).fuse(), o!()),
                requestsLogger: Logger::root(Self::generateConsoleDrain().filter_level(Level::Trace).fuse(), o!()),
            };
        } else {
            return Self {
                errorLogger: Logger::root(Discard, o!()),
                infoLogger: Logger::root(Self::generateConsoleDrain().filter_level(Level::Debug).fuse(), o!()),
                requestsLogger: Logger::root(Self::generateConsoleDrain().filter_level(Level::Trace).fuse(), o!()),
            };
        }
    }

    pub fn initFile(logDir: String, useEnvVar: bool) -> Self {
        if useEnvVar {
            let mut logLevel = Level::Debug;
            let initialLogLevel = var("NTX_LOG_LEVEL");
            if initialLogLevel.is_err() {
                panic!("Could not obtain log level from environment variable.");
            } else {
                let processedLogLevel = Level::from_str(initialLogLevel.unwrap().as_str());
                if processedLogLevel.is_err() {
                    panic!("Invalid log level specified in environment variable.");
                } else {
                    logLevel = processedLogLevel.unwrap();
                }
            }
            return Self {
                errorLogger: Logger::root(Self::generateFileDrain(logDir.clone(), "error".to_owned()).filter_level(Level::Error).fuse(), o!()),
                infoLogger: Logger::root(Self::generateFileDrain(logDir.clone(), "info".to_owned()).filter_level(logLevel).fuse(), o!()),
                requestsLogger: Logger::root(Self::generateFileDrain(logDir.clone(), "requests".to_owned()).filter_level(Level::Trace).fuse(), o!()),
            };
        } else {
            return Self {
                errorLogger: Logger::root(Self::generateFileDrain(logDir.clone(), "error".to_owned()).filter_level(Level::Error).fuse(), o!()),
                infoLogger: Logger::root(Self::generateFileDrain(logDir.clone(), "info".to_owned()).filter_level(Level::Debug).fuse(), o!()),
                requestsLogger: Logger::root(Self::generateFileDrain(logDir.clone(), "requests".to_owned()).filter_level(Level::Trace).fuse(), o!()),
            };
        }
    }

    fn logItemToLogger(&self, logText: String, loggerType: i32, logLevel: Level) {
        let logger = if loggerType == 0 { self.errorLogger.clone() } else if loggerType == 1 { self.infoLogger.clone() } else { self.requestsLogger.clone() };
        match logLevel {
            Level::Trace => { slog_trace!(logger, "{}", logText.as_str()); }
            Level::Debug => { slog_debug!(logger, "{}", logText.as_str()); }
            Level::Info => { slog_info!(logger, "{}", logText.as_str()); }
            Level::Warning => { slog_warn!(logger, "{}", logText.as_str()); }
            Level::Error => { slog_error!(logger, "{}", logText.as_str()); }
            Level::Critical => { slog_crit!(logger, "{}", logText.as_str()); }
        }
    }

    fn logItemInternal(&self, logText: String, logLevel: Level, useError: bool, useInfo: bool, useRequests: bool) {
        if useError {
            self.logItemToLogger(logText.clone(), 0, logLevel);
        } if useInfo {
            self.logItemToLogger(logText.clone(), 1, logLevel);
        } if useRequests {
            self.logItemToLogger(logText.clone(), 2, logLevel);
        }
    }

    fn logItem(&self, logText: String, logLevel: Level, isRequests: bool) {
        let mut finalText: String = OS.to_owned();
        finalText.push_str("\r\n");
        for variable in vars() {
            finalText.push_str(variable.0.as_str());
            finalText.push_str("=");
            finalText.push_str(variable.1.as_str());
            finalText.push_str("; ");
        }
        finalText.push_str("\r\n");
        finalText.push_str(logText.as_str());
        self.logItemInternal(finalText, logLevel, !isRequests, !isRequests, isRequests);
    }

    pub fn logText(&self, logText: String, logLevel: Level) {
        self.logItem(logText, logLevel, false);
    }

    pub fn logActixRequest(&self, logRequest: HttpRequest) {
        let mut requestString: String = logRequest.method().as_str().to_owned();
        requestString.push_str(" ");
        requestString.push_str(logRequest.uri().to_string().as_str());
        requestString.push_str("\r\n");
        for headerKey in logRequest.headers().keys() {
            requestString.push_str(logRequest.headers().get(headerKey.to_owned()).unwrap().to_str().unwrap());
            requestString.push_str("\r\n");
        }
        self.logItem(requestString, Level::Info, true);
    }

    pub fn logActixResponse(&self, logRequest: HttpResponse) {
        let mut responseString: String = logRequest.status().to_string();
        responseString.push_str("\r\n");
        for headerKey in logRequest.headers().keys() {
            responseString.push_str(logRequest.headers().get(headerKey.to_owned()).unwrap().to_str().unwrap());
            responseString.push_str("\r\n");
        }
        self.logItem(responseString, Level::Info, true);
    }

    pub fn logError(&self, logText: String) {
        self.logText(logText, Level::Error)
    }

    pub fn logWarning(&self, logText: String) {
        self.logText(logText, Level::Warning)
    }

    pub fn logInfo(&self, logText: String) {
        self.logText(logText, Level::Info)
    }

    pub fn logDebug(&self, logText: String) {
        self.logText(logText, Level::Debug)
    }

    pub fn logServerEventAsError(&self, logText: String) {
        let mut finalText: String = "Server Event - ".to_owned();
        finalText.push_str(logText.as_str());
        self.logError(finalText);
    }

    pub fn logServerEventAsWarning(&self, logText: String) {
        let mut finalText: String = "Server Event - ".to_owned();
        finalText.push_str(logText.as_str());
        self.logWarning(finalText);
    }

    pub fn logServerEventAsInfo(&self, logText: String) {
        let mut finalText: String = "Server Event - ".to_owned();
        finalText.push_str(logText.as_str());
        self.logInfo(finalText);
    }

    pub fn logServerEventAsDebug(&self, logText: String) {
        let mut finalText: String = "Server Event - ".to_owned();
        finalText.push_str(logText.as_str());
        self.logDebug(finalText);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::array::IntoIter;
    use std::iter::FromIterator;
    use actix_web::test::TestRequest;
    use std::path::Path;
    use std::fs::File;
    use actix_web::http::Method;

    #[test]
    fn performTests() {
        let logger1: LoggingService = LoggingService::initStdout(false);
        let logger2: LoggingService = LoggingService::initStdout(true);
        // For testing, NTX_LOG_LEVEL = "INFO"
        assert_eq!(logger1.infoLogger.is_debug_enabled(), true);
        assert_eq!(logger2.infoLogger.is_debug_enabled(), false);
        let mut baseDir: String = "C:\\Users\\user\\Documents\\".to_owned();
        baseDir.push_str("this-directory-does-not-exist\\THIS_DIRECTORY_DOES_NOT_EXIST\\");
        let logger3: LoggingService = LoggingService::initFile(baseDir.clone(), false);
        baseDir.push_str("2\\");
        let logger4: LoggingService = LoggingService::initFile(baseDir.clone(), true);
        assert_eq!(logger3.infoLogger.is_debug_enabled(), true);
        assert_eq!(logger4.infoLogger.is_debug_enabled(), false);
        logger3.logError("Error".to_owned());
        logger3.logWarning("Warn".to_owned());
        logger3.logInfo("Info".to_owned());
        logger3.logDebug("Debug".to_owned());
        logger3.logServerEventAsError("Server Error".to_owned());
        logger4.logWarning("Warn".to_owned());
        logger4.logInfo("Info".to_owned());
        logger4.logDebug("Debug".to_owned());
        let httpRequest: HttpRequest = TestRequest::with_header("content-type", "text/plain")
            .uri("https://www.google.com/")
            .method(Method::GET)
            .to_http_request();
        logger3.logActixRequest(httpRequest);
        let baseDir2: String = Path::new(baseDir.as_str()).parent().unwrap().to_str().unwrap().to_owned();
        let file1: String = Path::new(baseDir.as_str()).join("info.log").to_str().unwrap().to_owned();
        let file2: String = Path::new(baseDir.as_str()).join("error.log").to_str().unwrap().to_owned();
        let file3: String = Path::new(baseDir.as_str()).join("requests.log").to_str().unwrap().to_owned();
        let file4: String = Path::new(baseDir2.as_str()).join("info.log").to_str().unwrap().to_owned();
        let file5: String = Path::new(baseDir2.as_str()).join("error.log").to_str().unwrap().to_owned();
        let file6: String = Path::new(baseDir2.as_str()).join("requests.log").to_str().unwrap().to_owned();
        let mut buffer = String::new();
        let mut part1: File = File::open(file1).unwrap();
        let mut part2: File = File::open(file2).unwrap();
        let mut part3: File = File::open(file3).unwrap();
        let mut part4: File = File::open(file4).unwrap();
        let mut part5: File = File::open(file5).unwrap();
        let mut part6: File = File::open(file6).unwrap();
        part1.read_to_string(&mut buffer);
        println!("{}", buffer);
        buffer = String::new();
        part2.read_to_string(&mut buffer);
        println!("{}", buffer);
        buffer = String::new();
        part3.read_to_string(&mut buffer);
        println!("{}", buffer);
        buffer = String::new();
        part4.read_to_string(&mut buffer);
        println!("{}", buffer);
        buffer = String::new();
        part5.read_to_string(&mut buffer);
        println!("{}", buffer);
        buffer = String::new();
        part6.read_to_string(&mut buffer);
        println!("{}", buffer);
        buffer = String::new();
    }
}

fn main() {
//    println!("Hello, world!");
}
