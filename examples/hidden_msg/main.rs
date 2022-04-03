
mod args;
mod commands;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()>{
    let a = args::get_args();

    commands::parse(a)?;

    println!("hi");
    Ok(())
}
