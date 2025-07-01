// Constants for timing calculations
pub(crate) const CLOCK_SPEED_HZ: u64 = 125_000_000; // 125 MHz
pub(crate) const NS_PER_TICK: f64 = 1_000_000_000.0 / CLOCK_SPEED_HZ as f64; // Nanoseconds per tick
pub(crate) const OVERHEAD_TICKS: u32 = 5; // Estimated overhead from timer reads and GPIO operations

// Define ROM size to test
#[cfg(feature = "rom_2364")]
pub const MAX_ADDRESS: usize = 0x2000; // 8KB
#[cfg(feature = "rom_2332")]
pub const MAX_ADDRESS: usize = 0x1000; // 4KB
#[cfg(feature = "rom_2316")]
pub const MAX_ADDRESS: usize = 0x800; // 2KB

// Expected pattern (example)
/*
pub const EXPECTED_PATTERN: [u8; 12] = [
    b'p', b'i', b'e', b'r', b's', b'.', b'r', b'o', b'c', b'k', b's', b'\n',
];
*/
pub const EXPECTED_PATTERN: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7,
    8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47,
    48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 58, 59, 60, 61, 62, 63,
];

// Set up the address pins
#[cfg(feature = "rom_2364")]
pub const ADDR_PINS: [u8; 13] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
#[cfg(feature = "rom_2332")]
pub const ADDR_PINS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
#[cfg(feature = "rom_2316")]
pub const ADDR_PINS: [u8; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Set up the data pins
pub const DATA_PINS: [u8; 8] = [14, 15, 16, 17, 18, 19, 20, 21];

// Set up the CS pin(s)
pub const CS1_PIN: u8 = 13; // CS1 for 2364
#[cfg(any(feature = "rom_2332", feature = "rom_2316"))]
pub const CS2_PIN: u8 = 12; // CS2 for 2332 & 2316
#[cfg(feature = "rom_2316")]
pub const CS3_PIN: u8 = 11; // CS3 for 2316

// Set CS1 pin active state
#[cfg(feature = "cs1_active_low")]
pub const CS1_LOW: bool = true;
#[cfg(feature = "cs1_active_high")]
pub const CS1_LOW: bool = false;

#[cfg(feature = "cs2_active_low")]
pub const CS2_LOW: bool = true;
#[cfg(feature = "cs2_active_high")]
pub const CS2_LOW: bool = false;

#[cfg(feature = "cs3_active_low")]
pub const CS3_LOW: bool = true;
#[cfg(feature = "cs3_active_high")]
pub const CS3_LOW: bool = false;

pub const RESET_PIN: u8 = 22;   // Reset pin for the ROM

// Check feature settings

// Check if ROM type is provided
#[cfg(not(any(feature = "rom_2316", feature = "rom_2332", feature = "rom_2364")))]
compile_error!("Specify ROM type features: rom_2316, rom_2332, or rom_2364");

// For ROM 2364: Must have CS1 (either active high or low)
#[cfg(all(
    feature = "rom_2364",
    not(any(feature = "cs1_active_high", feature = "cs1_active_low"))
))]
compile_error!("ROM 2364 requires CS1 to be defined as either active high or low");

// For ROM 2332: Must have CS1 AND CS2 (each either active high or low)
#[cfg(all(
    feature = "rom_2332",
    not(
        all(
            any(feature = "cs1_active_high", feature = "cs1_active_low"),
            any(feature = "cs2_active_high", feature = "cs2_active_low")
        )
    )
))]
compile_error!(
    "ROM 2332 requires both CS1 AND CS2 to each be defined as either active high or low"
);

// For ROM 2316: Must have CS1, CS2, AND CS3 (each either active high or low)
#[cfg(all(
    feature = "rom_2316",
    not(
        all(
            any(feature = "cs1_active_high", feature = "cs1_active_low"),
            any(feature = "cs2_active_high", feature = "cs2_active_low"),
            any(feature = "cs3_active_high", feature = "cs3_active_low")
        )
    )
))]
compile_error!(
    "ROM 2316 requires CS1, CS2, AND CS3 to each be defined as either active high or low"
);

// Error if CS1 is defined as both active high and active low
#[cfg(all(feature = "cs1_active_high", feature = "cs1_active_low"))]
compile_error!("CS1 cannot be both active high and active low");

// Error if CS2 is defined as both active high and active low
#[cfg(all(feature = "cs2_active_high", feature = "cs2_active_low"))]
compile_error!("CS2 cannot be both active high and active low");

// Error if CS3 is defined as both active high and active low
#[cfg(all(feature = "cs3_active_high", feature = "cs3_active_low"))]
compile_error!("CS3 cannot be both active high and active low");
