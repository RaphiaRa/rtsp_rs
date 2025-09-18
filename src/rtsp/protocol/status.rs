use std::convert::TryFrom;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

/// RTSP Status codes
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Continue = 100,
    OK = 200,
    Created = 201,
    LowOnStorageSpace = 250,
    MultipleChoices = 300,
    MovedPermanently = 301,
    MovedTemporarily = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    RequestEntityTooLarge = 413,
    RequestURITooLarge = 414,
    UnsupportedMediaType = 415,
    ParameterNotUnderstood = 451,
    ConferenceNotFound = 452,
    NotEnoughBandwidth = 453,
    SessionNotFound = 454,
    MethodNotValidInThisState = 455,
    HeaderFieldNotValidForResource = 456,
    InvalidRange = 457,
    ParameterIsReadOnly = 458,
    AggregateOperationNotAllowed = 459,
    OnlyAggregateOperationAllowed = 460,
    UnsupportedTransport = 461,
    DestinationUnreachable = 462,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    RTSPVersionNotSupported = 505,
    OptionNotSupported = 551,
}

#[derive(Debug, Error)]
pub struct InvalidStatusError {
    status: u32,
}

impl InvalidStatusError {
    pub fn new(status: u32) -> Self {
        InvalidStatusError { status }
    }
}

impl fmt::Display for InvalidStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid status code {}", self.status)
    }
}

impl TryFrom<u32> for Status {
    type Error = InvalidStatusError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(Status::Continue),
            200 => Ok(Status::OK),
            201 => Ok(Status::Created),
            250 => Ok(Status::LowOnStorageSpace),
            300 => Ok(Status::MultipleChoices),
            301 => Ok(Status::MovedPermanently),
            302 => Ok(Status::MovedTemporarily),
            303 => Ok(Status::SeeOther),
            304 => Ok(Status::NotModified),
            305 => Ok(Status::UseProxy),
            400 => Ok(Status::BadRequest),
            401 => Ok(Status::Unauthorized),
            402 => Ok(Status::PaymentRequired),
            403 => Ok(Status::Forbidden),
            404 => Ok(Status::NotFound),
            405 => Ok(Status::MethodNotAllowed),
            406 => Ok(Status::NotAcceptable),
            407 => Ok(Status::ProxyAuthenticationRequired),
            408 => Ok(Status::RequestTimeout),
            410 => Ok(Status::Gone),
            411 => Ok(Status::LengthRequired),
            412 => Ok(Status::PreconditionFailed),
            413 => Ok(Status::RequestEntityTooLarge),
            414 => Ok(Status::RequestURITooLarge),
            415 => Ok(Status::UnsupportedMediaType),
            451 => Ok(Status::ParameterNotUnderstood),
            452 => Ok(Status::ConferenceNotFound),
            453 => Ok(Status::NotEnoughBandwidth),
            454 => Ok(Status::SessionNotFound),
            455 => Ok(Status::MethodNotValidInThisState),
            456 => Ok(Status::HeaderFieldNotValidForResource),
            457 => Ok(Status::InvalidRange),
            458 => Ok(Status::ParameterIsReadOnly),
            459 => Ok(Status::AggregateOperationNotAllowed),
            460 => Ok(Status::OnlyAggregateOperationAllowed),
            461 => Ok(Status::UnsupportedTransport),
            462 => Ok(Status::DestinationUnreachable),
            500 => Ok(Status::InternalServerError),
            501 => Ok(Status::NotImplemented),
            502 => Ok(Status::BadGateway),
            503 => Ok(Status::ServiceUnavailable),
            504 => Ok(Status::GatewayTimeout),
            505 => Ok(Status::RTSPVersionNotSupported),
            551 => Ok(Status::OptionNotSupported),
            _ => Err(InvalidStatusError::new(value)),
        }
    }
}

impl From<Status> for u32 {
    fn from(value: Status) -> Self {
        value as u32
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", u32::from(*self))?;
        match self {
            Status::Continue => write!(f, "Continue"),
            Status::OK => write!(f, "OK"),
            Status::Created => write!(f, "Created"),
            Status::LowOnStorageSpace => write!(f, "Low on Storage Space"),
            Status::MultipleChoices => write!(f, "Multiple Choices"),
            Status::MovedPermanently => write!(f, "Moved Permanently"),
            Status::MovedTemporarily => write!(f, "Moved Temporarily"),
            Status::SeeOther => write!(f, "See Other"),
            Status::NotModified => write!(f, "Not Modified"),
            Status::UseProxy => write!(f, "Use Proxy"),
            Status::BadRequest => write!(f, "Bad Request"),
            Status::Unauthorized => write!(f, "Unauthorized"),
            Status::PaymentRequired => write!(f, "Payment Required"),
            Status::Forbidden => write!(f, "Forbidden"),
            Status::NotFound => write!(f, "Stream Not Found"),
            Status::MethodNotAllowed => write!(f, "Method Not Allowed"),
            Status::NotAcceptable => write!(f, "Not Acceptable"),
            Status::ProxyAuthenticationRequired => write!(f, "Proxy Authentication Required"),
            Status::RequestTimeout => write!(f, "Request Timeout"),
            Status::Gone => write!(f, "Gone"),
            Status::LengthRequired => write!(f, "Length Required"),
            Status::PreconditionFailed => write!(f, "Precondition Failed"),
            Status::RequestEntityTooLarge => write!(f, "Request Entity Too Large"),
            Status::RequestURITooLarge => write!(f, "Request URI Too Large"),
            Status::UnsupportedMediaType => write!(f, "Unsupported Media Type"),
            Status::ParameterNotUnderstood => write!(f, "Parameter Not Understood"),
            Status::ConferenceNotFound => write!(f, "Conference Not Found"),
            Status::NotEnoughBandwidth => write!(f, "Not Enough Bandwidth"),
            Status::SessionNotFound => write!(f, "Session Not Found"),
            Status::MethodNotValidInThisState => write!(f, "Method Not Valid In This State"),
            Status::HeaderFieldNotValidForResource => {
                write!(f, "Header Field Not Valid For Resource")
            }
            Status::InvalidRange => write!(f, "Invalid Range"),
            Status::ParameterIsReadOnly => write!(f, "Parameter Is Read Only"),
            Status::AggregateOperationNotAllowed => write!(f, "Aggregate Operation Not Allowed"),
            Status::OnlyAggregateOperationAllowed => write!(f, "Only Aggregate Operation Allowed"),
            Status::UnsupportedTransport => write!(f, "Unsupported Transport"),
            Status::DestinationUnreachable => write!(f, "Destination Unreachable"),
            Status::InternalServerError => write!(f, "Internal Server Error"),
            Status::NotImplemented => write!(f, "Not Implemented"),
            Status::BadGateway => write!(f, "Bad Gateway"),
            Status::ServiceUnavailable => write!(f, "Service Unavailable"),
            Status::GatewayTimeout => write!(f, "Gateway Timeout"),
            Status::RTSPVersionNotSupported => write!(f, "RTSP Version Not Supported"),
            Status::OptionNotSupported => write!(f, "Option Not Supported"),
        }
    }
}

/// RTSP Status parsing error
/// Returned when the status code is not recognized
#[derive(Debug, Error)]
pub enum ParseStatusError {
    #[error(transparent)]
    InvalidStatus(#[from] InvalidStatusError),
    #[error("Failed to parse status code {0}")]
    ParseIntError(#[from] ParseIntError),
}

impl FromStr for Status {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Status, Self::Err> {
        Ok(Status::try_from(s.parse::<u32>()?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        let status = Status::try_from(200).unwrap();
        assert_eq!(status, Status::OK);
        let status = Status::try_from(404).unwrap();
        assert_eq!(status, Status::NotFound);
        let status = Status::try_from(500).unwrap();
        assert_eq!(status, Status::InternalServerError);
    }

    #[test]
    fn test_parse_status_str() {
        let status = Status::from_str("200").unwrap();
        assert_eq!(status, Status::OK);
        let status = Status::from_str("404").unwrap();
        assert_eq!(status, Status::NotFound);
        let status = Status::from_str("500").unwrap();
        assert_eq!(status, Status::InternalServerError);
    }
}
