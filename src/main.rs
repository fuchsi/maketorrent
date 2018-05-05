/*
    maketorrent
    Copyright (C) 2018  Daniel Müller

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

extern crate bip_metainfo;
#[macro_use] extern crate clap;
extern crate num_cpus;
extern crate pbr;

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use bip_metainfo::{MetainfoBuilder, PieceLength};
use bip_metainfo::error::{ParseResult};
use clap::{App, AppSettings, Arg};
use pbr::{ProgressBar, Units};

fn main() {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .global_setting(AppSettings::ArgRequiredElseHelp)
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .arg(Arg::with_name("announce")
            .short("a")
            .long("announce")
            .help("Announce URL.")
            .value_name("url")
            //.value_name("url,[<url>,...]")
            //.value_delimiter(",")
            .required(true))
        .arg(Arg::with_name("comment")
            .short("c")
            .long("comment")
            .help("Add a comment to the Torrent file.")
            .value_name("comment"))
        .arg(Arg::with_name("piece-length")
            .short("l")
            .long("piece-length")
            .help("Set the piece length to 2^n Bytes. [default: auto]")
            .value_name("n")
            .required(false))
        .arg(Arg::with_name("name")
            .short("n")
            .long("name")
            .help("Set the name of the Torrent file.[default: basename of the target]")
            .value_name("name"))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .help("Set the path and filename of the Torrent file.[default: <name>.torrent]")
            .value_name("filename"))
        .arg(Arg::with_name("private")
            .short("p")
            .long("private")
            .help("Set the private flag."))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Explain what is being done."))
        .arg(Arg::with_name("no-date")
            .short("d")
            .long("no-date")
            .help("Don't write the creation date."))
        .arg(Arg::with_name("threads")
            .short("t")
            .long("threads")
            .help("Number of threads to use for hashing.[default: number of logical cores]"))
        .arg(Arg::with_name("source")
            .value_name("source directory or filename")
            .required(true));

    let matches = app.get_matches();

    let announce_urls: Vec<_> = matches.values_of("announce").unwrap().collect();
    let comment = matches.value_of("comment");
    let mut piece_length: u32 = match matches.value_of("piece-length") {
        Some(l) => l.parse().expect("piece-length is not a number!"),
        None => 0,
    };
    let private = matches.is_present("private");
    let no_date = matches.is_present("no-date");
    let verbose = matches.is_present("verbose");
    let source = matches.value_of("source").unwrap();
    let threads: usize = match matches.value_of("threads") {
        Some(t) => t.parse().expect("threads is not a number!"),
        None => num_cpus::get(),
    };

    if piece_length > 25 {
        eprintln!("piece lengths > 32MiB are not supported!");
        process::exit(1);
    }

    let path = Path::new(source);
    if !path.exists() {
        eprintln!("file not found");
        process::exit(1);
    }
    let name = match matches.value_of("name") {
        Some(name) => name,
        None => path.file_stem().unwrap().to_str().unwrap(),
    };
    let output = match matches.value_of("output") {
        Some(output) => output.to_owned(),
        None => format!("{}.torrent", name),
    };

    let creator = format!("{}/{}", crate_name!(), crate_version!());

    // Tracker groups
    let trackers = announce_urls.into_iter().map(|v| v.to_owned()).collect();
    let groups: Vec<Vec<String>> = vec![trackers];
    let trackers = &groups[0];

    let mut builder = MetainfoBuilder::new()
        .set_created_by(Some(&creator))
        .set_comment(comment);
    if private {
        builder = builder.set_private_flag(Some(true));
    }
    if !no_date {
        builder = builder.set_creation_date(Some(time()));
    }
    builder = builder.set_main_tracker(trackers.get(0).map(|t| &t[..]));

    if trackers.len() > 1 {
        builder = builder.set_trackers(Some(&groups));
    }

    if piece_length == 0 {
        builder = builder.set_piece_length(PieceLength::OptBalanced);
        piece_length = 512 * 1024;
    } else {
        piece_length = 2u32.pow(piece_length);
        builder = builder.set_piece_length(PieceLength::Custom(piece_length as usize));
    }

    let f = match verbose {
        true => create_torrent,
        false => create_torrent_silent,
    };

    if verbose {
        println!("Options:");
        println!("  Announce URL:  {}", trackers[0]);
        println!("  Torrent Name:  {}", name);
        println!("  Metafile:      {}", output);
        println!("  Piece Length:  {}", piece_length);
        if let Some(comment) = comment {
            println!("  Comment:       {}", comment);
        }
        println!("  Private:       {}", if private { "yes" } else { "no" });
        println!("  Creation Date: {}", if no_date { "no" } else { "yes" });
        println!("  Threads:       {}", threads);
    }

    match f(builder, &path, threads, piece_length) {
        Ok(bytes) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            write!(handle, "writing metainfo file...").unwrap();
            handle.flush().unwrap();

            let mut output_file = File::create(output).unwrap();
            output_file.write_all(&bytes).unwrap();

            writeln!(handle, "done").unwrap();
            handle.flush().unwrap();
        },
        Err(e) => {
            eprintln!("Error With Input: {:?}", e);
            process::exit(1);
        }
    }
}

// get the current unix timestamp
fn time() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

fn create_torrent<S>(builder: MetainfoBuilder, source: S, threads: usize, _piece_length: u32) -> ParseResult<Vec<u8>>
    where S: AsRef<Path>
{
    let total_size = total_size(&source);
    let mut pb = ProgressBar::new(total_size);
    pb.format("╢▌▌░╟");
    pb.set_units(Units::Bytes);

    let mut prev_progress = 0;
    builder.build(threads, source, move |progress| {
        let whole_progress = (progress * (total_size as f64)) as u64;
        let delta_progress = whole_progress - prev_progress;

        if delta_progress > 0 {
            pb.add(delta_progress);
        }
        prev_progress = whole_progress;
    })
}

fn create_torrent_silent<S>(builder: MetainfoBuilder, source: S, threads: usize, piece_length: u32) -> ParseResult<Vec<u8>>
    where S: AsRef<Path>
{
    let total_size = total_size(&source);
    let pieces = total_size / piece_length as u64;

    let stdout = io::stdout();

    let res = builder.build(threads, source, move |progress| {
        let hashed = (progress * pieces as f64) as u64;

        let mut handle = stdout.lock();
        write!(handle, "Hashed {} of {} pieces\r", hashed, pieces).unwrap();
        handle.flush().unwrap();
    });
    println!();
    res
}

fn total_size<P: AsRef<Path>>(path: P) -> u64
{
    let mut size: u64 = 0;
    let path = path.as_ref();
    if path.is_file() {
        size = path.metadata().unwrap().len();
    } else {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            size += total_size(entry.path());
        }
    }

    size
}