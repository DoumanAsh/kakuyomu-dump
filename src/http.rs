#![allow(clippy::result_large_err)]
use std::io;
use core::{time, fmt};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub enum Error {
    StatusFailed(u16),
    Transport(ureq::Transport),
    Read(io::Error)
}

impl fmt::Display for Error {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StatusFailed(code) => fmt.write_fmt(format_args!("Request failed with status={code}")),
            Self::Transport(reason) => fmt.write_fmt(format_args!("Unable to connect: {reason}")),
            Self::Read(reason) => fmt.write_fmt(format_args!("Unable to read response: {reason}")),
        }
    }
}

impl From<ureq::Error> for Error {
    #[inline]
    fn from(value: ureq::Error) -> Self {
        match value {
            ureq::Error::Transport(error) => Self::Transport(error),
            ureq::Error::Status(code, _) => Self::StatusFailed(code),
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(value: io::Error) -> Self {
        Self::Read(value)
    }
}

pub trait FromResponse: Sized {
    fn read_response(resp: ureq::Response) -> Result<Self, Error>;
}

impl FromResponse for () {
    #[inline(always)]
    fn read_response(_: ureq::Response) -> Result<Self, Error> {
        Ok(())
    }
}

impl FromResponse for String {
    #[inline(always)]
    fn read_response(resp: ureq::Response) -> Result<Self, Error> {
        resp.into_string().map_err(Into::into)
    }
}

pub struct Client {
    inner: ureq::Agent,
    timeout: time::Duration,
}

impl Client {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: ureq::builder().try_proxy_from_env(true).redirects(5).user_agent(USER_AGENT).build(),
            timeout: time::Duration::from_secs(5),
        }
    }

    pub fn get<T: FromResponse>(&self, url: &str) -> Result<T, Error> {
        let response = self.inner.get(url).timeout(self.timeout).call()?;
        if response.status() != 200 {
            Err(Error::StatusFailed(response.status()))
        } else {
            T::read_response(response)
        }
    }
}
