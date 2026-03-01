kakuyomu-dump
==============

[![Rust](https://github.com/DoumanAsh/kakuyomu-dump/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/kakuyomu-dump/actions/workflows/rust.yml)

Provides utility to dump novel from https://kakuyomu.jp/

## Usage

```
Utility to download text of the kakuyomu novels

USAGE: [OPTIONS] <novel>

OPTIONS:
    -h,  --help                           Prints this help information
         --from <from>                    Specify from which chapter to start dumping. Default: 1.
         --to <to>                        Specify until which chapter to dump.
    -o,  --out <out>                      Output file name. By default writes ./<title>.md
         --rate <rate>                    Number of chapters to download at most per interval. Defaults to no limit.
         --rate_interval <rate_interval>  Interval between rated downloads. Defaults to 1 second.

ARGS:
    <novel>  Id of the novel to dump (e.g. 1177354054883819762)
```

## Convert to EPUB

I recommend to use [pandoc](https://github.com/jgm/pandoc):

```
pandoc --embed-resources --standalone --shift-heading-level-by=-1 --from=gfm -o novel.epub novel.md
```

## Unified CLI

This CLI was merged into my unified tool [shousetsu-dump](https://github.com/DoumanAsh/shousetsu-dump) with support for multiple sources to download novels from
