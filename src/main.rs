use kakuyomu_dump::*;

use std::{fs, path};
use std::process::ExitCode;

fn main() -> ExitCode {
    let stdio = stdio::Io::new();

    match cli::Cli::new() {
        Some(Ok(args)) => run(stdio, args),
        Some(Err(code)) => code,
        None => todo!(),
    }
}
fn construct_file_path(dir: &str, name: &str) -> path::PathBuf {
    let mut path = path::PathBuf::from(dir);
    path.push(name);
    path.set_extension("md");

    path
}

fn run(io: stdio::Io, args: cli::Cli) -> ExitCode {
    let http = http::Client::new();
    let mut stderr = io.stderr().ignore_errors();
    let mut stdout = io.stdout().ignore_errors();

    let novel_url = format!("https://kakuyomu.jp/works/{}", args.novel);
    let body: String = loop {
        stdout.write_fmt(format_args!(">>>{novel_url}: Fetch novel index..."));
        match http.get(&novel_url) {
            Ok(body) => {
                stdout.write_fmtn(format_args!("OK"));
                break body;
            }
            Err(http::Error::StatusFailed(404)) => {
                stdout.write_fmtn(format_args!("ERR"));
                stderr.write_fmtn(format_args!("No such novel found"));
                return ExitCode::FAILURE
            }
            Err(error) => {
                stdout.write_fmtn(format_args!("ERR"));
                stderr.write_fmtn(format_args!("{error}"));
                continue
            }
        };
    };

    let index = html::Document::new(&body);
    let index = match index.get_index() {
        Some(Ok(index)) => index,
        Some(Err(error)) => {
            stderr.write_fmtn(format_args!("Unable to deserialize chapter index: {error}"));
            return ExitCode::FAILURE
        }
        None => {
            stderr.write_fmtn(format_args!("Unable to fetch chapter index"));
            return ExitCode::FAILURE
        }
    };

    let mut url = String::new();
    let max = match args.to {
        Some(max) => if max.get() > index.chapters.len() {
            stderr.write_fmtn(format_args!("Novel has only {} chapters, but option -to is set to '{}'", index.chapters.len(), max));
            return ExitCode::FAILURE
        } else {
            max.get()
        },
        None => index.chapters.len()
    };

    let title = match index.title {
        Some(title) => {
            let title = html::Title::new(title);
            stdout.write_fmtn(format_args!("Title: {}", title.name));
            if let Some(author) = title.author {
                stdout.write_fmtn(format_args!("Author: {}", author));
            }
            title.name
        },
        None => {
            stderr.write_fmtn(format_args!("Unable to recognize novel's title"));
            return ExitCode::FAILURE
        }
    };

    let novel_file_name = match args.out {
        Some(out) => path::PathBuf::from(out),
        None => construct_file_path(".", title),
    };
    stdout.write_fmtn(format_args!("Number of chapters: {}", index.chapters.len()));

    let min = args.from.get();
    stdout.write_fmtn(format_args!("Download chapters: {}..{}", min, max));

    let mut novel_out = match fs::OpenOptions::new().create(true).write(true).truncate(true).open(&novel_file_name) {
        Ok(novel_out) => novel_out,
        Err(error) => {
            stderr.write_fmtn(format_args!("{}: Cannot write: {error}", novel_file_name.display()));
            return ExitCode::FAILURE
        }
    };

    macro_rules! write_novel {
        ($($arg:tt)*) => {
            if let Err(error) = std::io::Write::write_fmt(&mut novel_out, format_args!($($arg)*)) {
                stderr.write_fmtn(format_args!("{}: Cannot write: {error}", novel_file_name.display()));
                return ExitCode::FAILURE
            }
        };
    }

    write_novel!("{}\n===================\n", title);

    write_novel!("Original: {novel_url}\n");

    let max_idx = max - 1;
    let selectors = html::ChapterSelector::new();
    for (idx, chapter) in index.chapters.into_iter().enumerate().skip(min - 1) {
        if idx > max_idx {
            break;
        }

        url.push_str(&novel_url);
        url.push_str("/episodes/");
        url.push_str(&chapter);

        stdout.write_fmt(format_args!(">>>{url}: Downloading..."));
        let body: String = match http.get(&url) {
            Ok(body) => body,
            Err(error) => {
                stdout.write_fmt(format_args!("ERR"));
                stderr.write_fmtn(format_args!("{error}"));
                continue
            }
        };
        let chapter = html::Document::new(&body);
        let (title, body) = match chapter.get_chapter_content(&selectors) {
            Some(result) => result,
            None => {
                stdout.write_fmtn(format_args!("ERR"));
                stderr.write_fmtn(format_args!("!!!Cannot find chapter content"));
                return ExitCode::FAILURE
            }
        };
        stdout.write_fmtn(format_args!("OK"));
        match title {
            Some(title) => write_novel!("\n{title}\n-------------------\n"),
            None => write_novel!("\nChapter {}\n-------------------\n", idx + 1),
        }
        for line in body {
            match line {
                html::Line::Break => write_novel!("<br/>\n"),
                html::Line::Paragraph(line) => write_novel!("{}\n\n", line.inner_html()),
            }
        }

        url.clear();
    }

    if let Err(error) = std::io::Write::flush(&mut novel_out) {
        stderr.write_fmtn(format_args!("{}: Cannot write: {error}", novel_file_name.display()));
        return ExitCode::FAILURE
    }
    stdout.write_fmtn(format_args!("-------------------"));
    stdout.write_fmtn(format_args!("Output: {}", novel_file_name.display()));
    stdout.write_fmtn(format_args!("Pandoc command to generate EPUB:\npandoc --embed-resources --standalone --shift-heading-level-by=-1 --from=gfm -o novel.epub \"{}\"", novel_file_name.display()));
    ExitCode::SUCCESS
}
