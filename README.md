# maketorrent

A Bittorrent meta file generator

## Usage
```
USAGE:
    maketorrent [OPTIONS] <source directory or filename> --announce <url>

OPTIONS:
    -a, --announce <url>       Announce URL.
    -c, --comment <comment>    Add a comment to the Torrent file.
    -h, --help                 Prints help information
    -n, --name <name>          Set the name of the Torrent file.[default: basename of the target]
    -d, --no-date              Don't write the creation date.
    -o, --output <filename>    Set the path and filename of the Torrent file.[default: <name>.torrent]
    -l, --piece-length <n>     Set the piece length to 2^n Bytes. [default: auto]
    -p, --private              Set the private flag.
    -t, --threads              Number of threads to use for hashing.[default: number of logical cores]
    -V, --version              Prints version information
    -v, --verbose              Explain what is being done.

ARGS:
    <source directory or filename> 
```

## Installation
```bash
cargo install maketorrent
```
or
```bash
git clone https://github.com/fuchsi/maketorrent.git
cd maketorrent
cargo install
```