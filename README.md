kakuyomu-dump
==============

![](https://github.com/DoumanAsh/kakuyomu-dump/workflows/Rust/badge.svg)

Provides utility to dump novel from https://kakuyomu.jp/

## Usage

```
Utility to download text of the kakuyomu novels

USAGE: [OPTIONS] <novel>

OPTIONS:
    -h,  --help         Prints this help information
         --from <from>  Specify from which chapter to start dumping. Default: 1.
         --to <to>      Specify until which chapter to dump.
    -o,  --out <out>    Output file name. By default writes ./<title>.md

ARGS:
    <novel>  Id of the novel to dump (e.g. 1177354054883819762)
```

## Convert to EPUB

I recommend to use [pandoc](https://github.com/jgm/pandoc):

```
pandoc --embed-resources --standalone --shift-heading-level-by=-1 --from=gfm -o novel.epub novel.md
```
