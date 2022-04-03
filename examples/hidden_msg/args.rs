use clap::{arg, Command, ArgMatches};

pub fn get_args() -> ArgMatches {
    Command::new("NicePics")
        .version("1.0")
        .author("Vulpesx")
        .about("messes with png files")
        .subcommand(
            Command::new("encode")
            .about("encodes a message into the png")
            .alias("e")
            .arg(arg!(-f --file <FILE> "path to png"))
            .arg(arg!(-c --chunk <CHUNK> "the chunk type: 4 characters"))
            .arg(arg!(-m --msg <MSG> "the message"))
            .arg(arg!(-o --output <FILE> "the output file").required(false)))
        .subcommand(
            Command::new("decode")
            .about("decode a png file")
            .alias("d")
            .arg(arg!(-f --file <FILE> "path to png"))
            .arg(arg!(-c --chunk <CHUNK> "the chunk type with the message")))
        .subcommand(
            Command::new("remove")
            .about("remove a message from a png file")
            .alias("r")
            .arg(arg!(-f --file <FILE> "path to png"))
            .arg(arg!(-c --chunk <CHUNK> "the chunk type to remove")))
        .subcommand(
            Command::new("print")
            .about("print a message")
            .alias("p")
            .arg(arg!(-f --file <FILE> "path to png")))
        .get_matches()
}
