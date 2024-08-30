use core::fmt;
use std::borrow::Cow;

use scraper::html::Html;
use scraper::selector::Selector;
use serde_ignored_type::IgnoredAny;

pub struct Title<'a> {
    pub name: &'a str,
    pub author: Option<&'a str>,
}

impl<'a> Title<'a> {
    pub fn new(mut title: &'a str) -> Self {
        const AUTHOR_END: char = '）';
        const AUTHOR_START: char = '（';

        title = title.trim();
        if let Some(stripped) = title.strip_suffix(" - カクヨム") {
            title = stripped;
        }
        let author = match title.rfind(AUTHOR_END) {
            Some(idx) => {
                let mut author = &title[..idx];
                //Make sure we have opening bracket
                match author.rfind(AUTHOR_START) {
                    Some(mut start_idx) => {
                        //If author used brackets in his name, then we need to account for that
                        //So we count number of round brackets
                        let mut sub_end_count = 0usize;
                        let mut author_sub = author;
                        while let Some(nested_idx) = author_sub.rfind(AUTHOR_END) {
                            sub_end_count = sub_end_count.saturating_add(1);
                            author_sub = &author_sub[..nested_idx];
                        }

                        //if there is nested closing brackets, then skip equal number of opening brackets
                        while sub_end_count > 0 {
                            if let Some(new_idx) = author[..start_idx - AUTHOR_START.len_utf8()].rfind(AUTHOR_START) {
                                sub_end_count -= 1;
                                start_idx = new_idx;
                            } else {
                                break;
                            }
                        }
                        author = &author[start_idx + AUTHOR_START.len_utf8()..];
                        title = &title[..start_idx];

                        Some(author)
                    },
                    None => None
                }
            },
            None => None,
        };

        Self {
            name: title,
            author,
        }
    }
}

#[derive(Debug, serde_derive::Deserialize)]
struct ScriptState {
    props: Props,
}

#[allow(non_snake_case)]
#[derive(Debug, serde_derive::Deserialize)]
struct Props {
    pageProps: PageProps,
}

#[allow(non_snake_case)]
#[derive(Debug, serde_derive::Deserialize)]
struct PageProps {
    __APOLLO_STATE__: ApolloState
}

#[derive(Debug)]
struct ApolloState {
    chapters: Vec<String>,
}

struct ApolloStateVisitor;

impl<'de> serde::de::Visitor<'de> for ApolloStateVisitor {
    type Value = ApolloState;
    #[inline(always)]
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("Expected __APOLLO_STATE__ to contain JSON")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        const SUFFIX: &str = "Episode:";

        let mut chapters = Vec::new();
        while let Some(entry) = map.next_key::<Cow<'de, str>>()? {
            if let Some(chapter) = entry.strip_prefix(SUFFIX) {
                chapters.push(chapter.to_owned());
            }
            let _ = map.next_value::<IgnoredAny>();
        }

        Ok(ApolloState {
            chapters
        })
    }
}

impl<'de> serde::Deserialize<'de> for ApolloState {
    #[inline(always)]
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ApolloStateVisitor)
    }
}

#[derive(Debug)]
pub struct Index<'a> {
    pub title: Option<&'a str>,
    pub chapters: Vec<String>
}

pub struct ChapterSelector {
    body: Selector,
    line: Selector,
    title: Selector,
}

impl ChapterSelector {
    pub fn new() -> Self {
        Self {
            body: Selector::parse(".widget-episodeBody.js-episode-body").unwrap(),
            line: Selector::parse("p").unwrap(),
            title: Selector::parse(".widget-episodeTitle").unwrap()
        }
    }
}

pub enum Line<'a> {
    Paragraph(scraper::ElementRef<'a>),
    Break,
}

impl<'a> Line<'a> {
    #[inline(always)]
    fn new(line: scraper::ElementRef<'a>) -> Self {
        match line.attr("class") {
            Some("blank") => Self::Break,
            _ => Self::Paragraph(line),
        }
    }
}

pub struct Document {
    inner: Html,
}

impl Document {
    pub fn new(html: &str) -> Self {
        Self {
            inner: Html::parse_document(html)
        }
    }

    pub fn get_index(&self) -> Option<Result<Index<'_>, serde_json::Error>> {
        let title = Selector::parse("title").unwrap();
        let title = self.inner.select(&title).next().and_then(|title| {
            title.text().next()
        });

        let selector = Selector::parse("script").unwrap();
        for elem in self.inner.select(&selector) {
            let element = elem.value();
            match (element.attr("type"), element.attr("id")) {
                (Some("application/json"), Some("__NEXT_DATA__")) => {
                    if let Some(json) = elem.text().next() {
                        return Some(serde_json::from_str::<ScriptState>(json).map(|result| Index {
                            title,
                            chapters: result.props.pageProps.__APOLLO_STATE__.chapters
                        }));
                    } else {
                        continue;
                    }
                },
                _ => (),
            }
        }
        None
    }

    pub fn get_chapter_content<'a>(&'a self, selectors: &'a ChapterSelector) -> Option<(Option<String>, impl Iterator<Item = Line> + 'a)> {
        let title = self.inner.select(&selectors.title).next().map(|html| html.inner_html());

        if let Some(body) = self.inner.select(&selectors.body).next() {
            Some((title, body.select(&selectors.line).map(Line::new)))
        } else {
            None
        }
    }
}
