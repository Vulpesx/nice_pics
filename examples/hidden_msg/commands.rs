use std::{fs, io::{Read, Write}, str::FromStr};

use nice_pics::prelude::*;
use anyhow::{bail, Result};
use clap::ArgMatches;

type Error = anyhow::Error;

pub enum Commands {
    ENCODE,
    DECODE,
    REMOVE,
    PRINT,
}

impl Commands {
    pub fn from_cmd(&self) -> &str {
        match self {
            Commands::ENCODE => "encode",
            Commands::DECODE => "decode",
            Commands::REMOVE => "remove",
            Commands::PRINT => "print",
        }
    }

    pub fn to_cmd(s: &str) -> Commands {
        match s {
            "encode" => Commands::ENCODE,
            "decode" => Commands::DECODE,
            "remove" => Commands::REMOVE,
            "print" => Commands::PRINT,
            _ => Commands::PRINT,
        }
    }

    pub fn alias(&self) -> &str {
        match self {
            Commands::ENCODE => "e",
            Commands::DECODE => "d",
            Commands::REMOVE => "r",
            Commands::PRINT => "p",
        }
    }
}

pub fn parse(args: ArgMatches) -> Result<(), Error> {
    let command = args.subcommand();
    if let Some((c, args)) = command {
        let c = Commands::to_cmd(c);
        match c {
            Commands::ENCODE => encode(args)?,
            Commands::DECODE => decode(args)?,
            Commands::REMOVE => remove(args)?,
            Commands::PRINT => print(args)?,
        }
    } else {
        bail!("no subcommand used");
    }
    Ok(())
}

fn encode(args: &ArgMatches) -> Result<(), Error> {
    // ask if user wants to continue as data may be overiden
    let mut usrin = String::new();
    println!("this will remove any existing messages using the same chunk_type? [y/n]:");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut usrin).unwrap();
    if usrin.to_lowercase().contains("n") { bail!("user didnt want to continue"); }

    let f = args.value_of("file").unwrap();
    let mut p = read_file(f)?;
    p.remove_chunk("IEND");// end chunk removed as we can only append
    let ct = args.value_of("chunk").unwrap(); //chunk_type
    p.remove_chunk(ct); // do not return err as it doesnt matter if chunk exists
    let m = args.value_of("msg").unwrap();
    let c = Chunk::new(ChunkType::from_str(ct)?, m.bytes().collect());
    p.append_chunk(c);
    p.append_chunk(Chunk::new(ChunkType::from_str("IEND")?, Vec::new()));

    let o = args.value_of("output");

    let mut f = if o.is_some() { fs::File::create(o.unwrap())? } else { fs::File::create(f)? };
    f.write_all(p.as_bytes().as_ref())?;
    f.flush()?;

    Ok(())
}

fn decode(args: &ArgMatches) -> Result<(), Error> {
    let f = args.value_of("file").unwrap();
    let p = read_file(f)?;
    let ct = args.value_of("chunk").unwrap();
    let c = p.chunk_by_type(ct);
    if let Some(c) = c {
        println!("bytes {:?}", c.data());
        println!("msg: {}", c.data_as_string()?);
    } else {
        bail!("no message or wrong chunk type")
    }
    Ok(())
}

fn remove(args: &ArgMatches) -> Result<(), Error> {
    let f = args.value_of("file").unwrap();
    let mut p = read_file(f)?;
    let ct = args.value_of("chunk").unwrap();
    p.remove_chunk(ct)?;
    let mut f = fs::File::create(f)?;
    f.write_all(p.as_bytes().as_ref())?;
    f.flush()?;

    Ok(())
}

fn print(args: &ArgMatches) -> Result<(), Error> {
    let f = args.value_of("file").unwrap();
    let p = read_file(f)?;
    println!("{:?}", p.as_bytes());

    Ok(())
}

fn read_file(p: &str) -> Result<Png, Error> {
    println!("reading {}", p);
    let mut f = fs::File::open(p).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf);

    Png::try_from(buf.as_ref())
}

