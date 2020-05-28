use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "gbjit")]
#[structopt(about = r#"
A WIP just-in-time compiler for the GameBoy and GameBoy Colour.

Currently just disassembles a given binary.
"#)]
pub struct Args {
    /// GB bios file
    pub bios: String,

    /// GB rom to run
    pub rom: String,

    /// Logfile to write GB and x86 disassembly to
    #[structopt(short, long)]
    pub disassembly_logfile: Option<String>,

    /// Whether to generate log traces for each instruction executed
    #[structopt(short, long)]
    pub trace_pc: bool,

    /// Whether to use a standardized logging format for execution diffing
    #[structopt(long = "std-logging", requires = "trace-pc")]
    pub std_logging: bool,

    #[structopt(
        short = "p",
        long = "px",
        default_value = "960,864",
        parse(try_from_str = parse_tuple)
    )]
    pub screen_dimensions: (u32, u32),

    /// Only advance the frame when the 'n' key is hit
    #[structopt(short, long)]
    pub wait: bool,

    /// Whether to run in headless mode, where the gb is emulated with no IO, just to generate logs
    #[structopt(short = "H", long)]
    pub headless: bool,
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse {src}")]
struct DimensionParseError {
    src: String,
}

impl From<&str> for DimensionParseError {
    fn from(s: &str) -> Self {
        DimensionParseError {
            src: String::from(s),
        }
    }
}

fn parse_tuple(src: &str) -> Result<(u32, u32), DimensionParseError> {
    use std::str::FromStr;

    let components: Result<Vec<u32>, std::num::ParseIntError> =
        src.split(",").map(u32::from_str).collect();
    let components = components.map_err(|_| DimensionParseError::from(src))?;
    match *components {
        [w, h] => Ok((w, h)),
        _ => Err(src.into()),
    }
}
