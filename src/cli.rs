use arg::Args;

use std::env;
use core::num::NonZeroUsize;
use std::process::ExitCode;

pub fn default_from_value() -> NonZeroUsize {
    unsafe {
        core::num::NonZeroUsize::new_unchecked(1)
    }
}

#[derive(Args, Debug)]
///Utility to download text of the kakuyomu novels
pub struct Cli {
    #[arg(long, default_value = "default_from_value()")]
    ///Specify from which chapter to start dumping. Default: 1.
    pub from: NonZeroUsize,
    #[arg(long)]
    ///Specify until which chapter to dump.
    pub to: Option<NonZeroUsize>,
    #[arg(long, short)]
    ///Output file name. By default writes ./<title>.md
    pub out: Option<String>,
    #[arg(required)]
    ///Id of the novel to dump (e.g. 1177354054883819762)
    pub novel: String,
}

impl Cli {
    #[inline]
    pub fn new() -> Option<Result<Self, ExitCode>> {
        let args: Vec<_> = env::args().skip(1).collect();

        if args.is_empty() {
            return None;
        }

        match Self::from_args(args.iter().map(String::as_str)) {
            Ok(args) => Some(Ok(args)),
            Err(arg::ParseKind::Sub(name, arg::ParseError::HelpRequested(help))) => {
                println!("{name}: {}", help);
                Some(Err(ExitCode::SUCCESS))
            },
            Err(arg::ParseKind::Top(arg::ParseError::HelpRequested(help))) => {
                println!("{}", help);
                Some(Err(ExitCode::SUCCESS))
            },
            Err(error) => {
                eprintln!("{}", error);
                Some(Err(ExitCode::FAILURE))
            }
        }
    }
}
