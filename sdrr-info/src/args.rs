use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Args {
    pub command: Command,
    pub firmware: PathBuf,
    pub set: Option<u8>,
    pub addr: Option<u32>,
    pub range: Option<(u32, u32)>,
    pub cs1: Option<bool>,
    pub cs2: Option<bool>,
    pub cs3: Option<bool>,
    pub x1: Option<bool>,
    pub x2: Option<bool>,
    pub output_mangled: Option<bool>,
    pub output_binary: Option<bool>,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Info,
    LookupRaw,
    Lookup,
}

#[derive(Parser)]
#[command(name = "sdrr-info")]
#[command(about = "Software Defined Retro ROM firmware information")]
#[command(version)]
#[command(after_help = "Use 'help <subcommand>' for detailed options (e.g., 'help lookup')")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    firmware: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Outputs comprehensive information about the SDRR firmware, including:
    /// - Build and hardware properties
    /// - Firmware configuration options
    /// - Embedded ROM image information
    #[command(verbatim_doc_comment)]
    Info {
        /// Firmware filename (.bin or .elf files supported)
        firmware: PathBuf,
    },
    /// Lookup a byte associated with a raw STM32F4 address port line
    /// configuration.  Use this to detect what byte the STM32F4 will output
    /// on the data lines in response to a particular 16-bit input from the
    /// appropriate port (port C on stock SDRR 24-pin hardware revisions).
    /// 
    /// The output byte can be demangled (i.e. the correct value), which is the
    /// default, or mangled (i.e. the raw value as it will be written to the
    /// STM32F4 data port, port A on stock SDRR 24-pin hardware revisions).
    LookupRaw {
        /// Firmware filename (.bin or .elf files supported)
        firmware: PathBuf,
        /// ROM set number (starts from 0)
        #[arg(short, long, default_value="0")]
        set: u8,
        /// Address to look up (in hex, e.g., 0x1000 or $1000)
        #[arg(short, long, value_parser = parse_hex)]
        addr: u32,
        /// Output mangled data byte(s)
        /// (not specifying this outputs a demangled byte)
        #[arg(long, default_value="false", verbatim_doc_comment)]
        output_mangled: bool,
    },
    /// Lookup a byte associated with an actual address lookup on the address
    /// bus, using a non-mangled address.  Use this to detect what byte the
    /// STM32F4 will output on the data lines in response to a particular
    /// address on the address bus.  This option requires CS line states,
    /// including, for multi-rom sets, the X1 and X2 pins. 
    /// 
    /// This option allows a single byte or range of bytes to be looked up.
    /// The output can be output as text (2 byte hex values) or optionally
    /// output as binary data.
    /// 
    /// Outputting a range as binary data can be useful if you want to compare
    /// an entire stored ROM image stored in the firmware with the original
    /// file.
    Lookup {
        /// Firmware filename (.bin or .elf files supported)
        firmware: PathBuf,
        /// ROM set number (starts from 0)
        #[arg(short, long, default_value="0")]
        set: u8,
        /// Address to look up (in hex, e.g., 0x1000 or $1000)
        #[arg(short, long, value_parser = parse_hex)]
        addr: Option<u32>,
        /// Address range to look up (in hex, e.g., 0x1000-1FFF)
        #[arg(short, long, value_parser = parse_range)]
        range: Option<(u32, u32)>,
        /// CS1 line state (0 or 1) (default: 0)
        #[arg(long, default_value = "0", value_parser = parse_cs_line)]
        cs1: u8,
        /// CS2 line state (0 or 1) - valid for 2332/2316 ROMs only
        #[arg(long, value_parser = parse_cs_line)]
        cs2: Option<u8>,
        /// CS3 line state (0 or 1) - valid for 2316 ROMs only
        #[arg(long, value_parser = parse_cs_line)]
        cs3: Option<u8>,
        /// X1 line state (0 or 1) - valid for multi-ROM sets only
        #[arg(long, value_parser = parse_cs_line)]
        x1: Option<u8>,
        /// X2 line state (0 or 1) - valid for multi-ROM sets only
        /// #[arg(long, value_parser = parse_cs_line)]
        x2: Option<u8>,
        /// Output mangled data byte(s)
        /// (not specifying this outputs a demangled byte)
        #[arg(long, default_value="false", verbatim_doc_comment)]
        output_mangled: bool,
        /// Output binary data instead of text
        /// (default: false = text output)
        #[arg(long, default_value="false", verbatim_doc_comment)]
        output_binary: bool,
    },
}

fn parse_hex(s: &str) -> Result<u32, String> {
    let cleaned = if s.starts_with("0x") || s.starts_with("0X") {
        &s[2..]
    } else if s.starts_with('$') {
        &s[1..]
    } else {
        s
    };
    
    u32::from_str_radix(cleaned, 16)
        .map_err(|_| format!("Invalid hex value: {}", s))
}

fn parse_cs_line(s: &str) -> Result<u8, String> {
    match s {
        "0" => Ok(0),
        "1" => Ok(1),
        _ => Err("CS line state must be 0 or 1".to_string()),
    }
}

fn parse_range(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Range format must be: start-end (e.g., 1000-1FFF)".to_string());
    }
    
    let start = parse_hex(parts[0])?;
    let end = parse_hex(parts[1])?;
    
    if start > end {
        return Err("Range start must be less than or equal to end".to_string());
    }
    
    Ok((start, end))
}

pub fn parse_args() -> Result<Args, String> {
    let cli = Cli::parse();
    
    let (command, firmware, set, addr, range, cs1, cs2, cs3, x1, x2, output_mangled, output_binary) = match cli.command {
        Some(Commands::Info { firmware }) => {
            (Command::Info, firmware, None, None, None, None, None, None, None, None, None, None)
        }
        
        Some(Commands::LookupRaw { 
            firmware, set, addr, output_mangled
        }) => {
            
            (Command::LookupRaw, firmware, Some(set), Some(addr), None, 
             None, None, None, None, None, Some(output_mangled), None)
        }
        
        Some(Commands::Lookup { 
            firmware, set, addr, range, cs1, cs2, cs3, x1, x2, output_mangled, output_binary
        }) => {
            if addr.is_some() && range.is_some() {
                return Err("Cannot specify both --addr and --range".to_string());
            }
            if addr.is_none() && range.is_none() {
                return Err("Must specify either --addr or --range".to_string());
            }
            
            (Command::Lookup, firmware, Some(set), addr, range, 
             Some(cs1), cs2, cs3, x1, x2, Some(output_mangled), Some(output_binary))
        }
        
        None => {
            if let Some(firmware) = cli.firmware {
                (Command::Info, firmware, None, None, None, None, None, None, None, None, None, None)
            } else {
                return Err("No firmware file specified. Use --help for usage information.".to_string());
            }
        }
    };
    
    // Validate firmware file exists
    if !firmware.exists() {
        return Err(format!("Firmware file does not exist: {}", firmware.display()));
    }
    
    // Validate firmware file extension
    if let Some(ext) = firmware.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if ext_str != "bin" && ext_str != "elf" {
            return Err("Firmware file must have .bin or .elf extension".to_string());
        }
    } else {
        return Err("Firmware file must have .bin or .elf extension".to_string());
    }

    // Validate the address/range
    if let Some(addr) = addr {
        if addr > 0xFFFF {
            return Err("Address must be in the range 0x0000 to 0xFFFF".to_string());
        }
    }
    if let Some(range) = range {
        if range.0 > 0xFFFF || range.1 > 0xFFFF {
            return Err("Range must be in the range 0x0000 to 0xFFFF".to_string());
        }
    }
    
    Ok(Args {
        command,
        firmware,
        set,
        addr,
        range,
        cs1: cs1.map(|c| c != 0),
        cs2: cs2.map(|c| c != 0),
        cs3: cs3.map(|c| c != 0),
        x1: x1.map(|c| c != 0),
        x2: x2.map(|c| c != 0),
        output_mangled,
        output_binary,
    })
}
