use crate::constants::{CLOCK_SPEED_HZ, NS_PER_TICK, OVERHEAD_TICKS};
use defmt::info;

/// Integer square root using binary search - no_std compatible
fn isqrt(n: u64) -> u32 {
    if n == 0 || n == 1 {
        return n as u32;
    }

    let mut left = 1u32;
    let mut right = core::cmp::min(n as u32, u32::MAX / 2);
    let mut result = 0;

    while left <= right {
        let mid = left + (right - left) / 2;
        let mid_squared = (mid as u64) * (mid as u64);

        if mid_squared == n {
            return mid;
        }

        if mid_squared < n {
            left = mid + 1;
            result = mid; // Update result to the floor value
        } else {
            right = mid - 1;
        }
    }

    result
}

/// Calculate and log statistics for both timing results
pub fn calculate_timing_statistics(
    timing_low: &[u32],  // Time for D7 to go low (read time)
    timing_high: &[u32], // Time for D7 to go high (reset time)
    max_address: usize,
) {
    if max_address == 0 {
        info!("No timing data to analyze");
        return;
    }

    info!("=== ROM Access Timing Statistics ===");
    info!("Sample size: {} addresses", max_address);

    // Process read time (D7 going low)
    let mut min_read = u32::MAX;
    let mut max_read = 0;
    let mut sum_read: u64 = 0;

    // Process reset time (D7 going high)
    let mut min_reset = u32::MAX;
    let mut max_reset = 0;
    let mut sum_reset: u64 = 0;

    // First pass - collect basic stats for both arrays
    for i in 0..max_address {
        // Process read time
        let read_ticks = timing_low[i].saturating_sub(OVERHEAD_TICKS);
        min_read = min_read.min(read_ticks);
        max_read = max_read.max(read_ticks);
        sum_read += read_ticks as u64;

        // Process reset time
        let reset_ticks = timing_high[i].saturating_sub(OVERHEAD_TICKS);
        min_reset = min_reset.min(reset_ticks);
        max_reset = max_reset.max(reset_ticks);
        sum_reset += reset_ticks as u64;
    }

    // Calculate averages
    let avg_read = sum_read / max_address as u64;
    let avg_reset = sum_reset / max_address as u64;

    // Second pass - calculate variance
    let mut sum_squared_diff_read: u64 = 0;
    let mut sum_squared_diff_reset: u64 = 0;

    for i in 0..max_address {
        // Read time variance calc
        let read_ticks = timing_low[i].saturating_sub(OVERHEAD_TICKS) as i64;
        let diff_read = read_ticks - avg_read as i64;
        sum_squared_diff_read += (diff_read * diff_read) as u64;

        // Reset time variance calc
        let reset_ticks = timing_high[i].saturating_sub(OVERHEAD_TICKS) as i64;
        let diff_reset = reset_ticks - avg_reset as i64;
        sum_squared_diff_reset += (diff_reset * diff_reset) as u64;
    }

    // Calculate standard deviations
    let variance_read = if max_address > 1 {
        sum_squared_diff_read / (max_address as u64 - 1)
    } else {
        0
    };

    let variance_reset = if max_address > 1 {
        sum_squared_diff_reset / (max_address as u64 - 1)
    } else {
        0
    };

    let std_dev_read = isqrt(variance_read);
    let std_dev_reset = isqrt(variance_reset);

    // Convert to nanoseconds
    let min_read_ns = (min_read as u64) * (NS_PER_TICK as u64);
    let max_read_ns = (max_read as u64) * (NS_PER_TICK as u64);
    let avg_read_ns = avg_read * (NS_PER_TICK as u64);
    let std_dev_read_ns = (std_dev_read as u64) * (NS_PER_TICK as u64);

    let min_reset_ns = (min_reset as u64) * (NS_PER_TICK as u64);
    let max_reset_ns = (max_reset as u64) * (NS_PER_TICK as u64);
    let avg_reset_ns = avg_reset * (NS_PER_TICK as u64);
    let std_dev_reset_ns = (std_dev_reset as u64) * (NS_PER_TICK as u64);

    // Calculate combined read+reset cycle time
    let min_cycle = min_read + min_reset;
    let max_cycle = max_read + max_reset;
    let avg_cycle = (avg_read + avg_reset) as u32;

    let min_cycle_ns = (min_cycle as u64) * (NS_PER_TICK as u64);
    let max_cycle_ns = (max_cycle as u64) * (NS_PER_TICK as u64);
    let avg_cycle_ns = (avg_cycle as u64) * (NS_PER_TICK as u64);

    // Log basic statistics for read time
    info!("--- ROM Read Time (D7 going low) ---");
    info!("Min: {} ticks ({} ns)", min_read, min_read_ns);
    info!("Max: {} ticks ({} ns)", max_read, max_read_ns);
    info!("Avg: {} ticks ({} ns)", avg_read, avg_read_ns);
    info!("StdDev: {} ticks ({} ns)", std_dev_read, std_dev_read_ns);

    // Log basic statistics for reset time
    info!("--- ROM Reset Time (D7 going high) ---");
    info!("Min: {} ticks ({} ns)", min_reset, min_reset_ns);
    info!("Max: {} ticks ({} ns)", max_reset, max_reset_ns);
    info!("Avg: {} ticks ({} ns)", avg_reset, avg_reset_ns);
    info!("StdDev: {} ticks ({} ns)", std_dev_reset, std_dev_reset_ns);

    // Log cycle statistics (read + reset)
    info!("--- Total Cycle Time (Read + Reset) ---");
    info!("Min cycle: {} ticks ({} ns)", min_cycle, min_cycle_ns);
    info!("Max cycle: {} ticks ({} ns)", max_cycle, max_cycle_ns);
    info!("Avg cycle: {} ticks ({} ns)", avg_cycle, avg_cycle_ns);

    // Performance metrics
    if min_read > 0 {
        // Max frequency based on minimum read time
        let max_freq_hz = CLOCK_SPEED_HZ / min_read as u64;
        let _max_freq_mhz = max_freq_hz / 1_000_000;
        let _max_freq_mhz_frac = ((max_freq_hz % 1_000_000) * 100) / 1_000_000;

        // Not convinced by this
        //info!(
        //    "Max theoretical read frequency: {}.{:02} MHz",
        //    max_freq_mhz, max_freq_mhz_frac
        //);
    }

    if min_cycle > 0 {
        // Max frequency based on minimum cycle time (read+reset)
        let max_cycle_freq_hz = CLOCK_SPEED_HZ / min_cycle as u64;
        let _max_cycle_freq_mhz = max_cycle_freq_hz / 1_000_000;
        let _max_cycle_freq_mhz_frac = ((max_cycle_freq_hz % 1_000_000) * 100) / 1_000_000;

        // Not convinced by this
        //info!(
        //    "Max theoretical cycle frequency: {}.{:02} MHz",
        //    max_cycle_freq_mhz, max_cycle_freq_mhz_frac
        //);
    }

    // Calculate and log coefficient of variation
    if avg_read > 0 {
        let cv_read = ((std_dev_read as u64) * 10000) / avg_read;
        info!(
            "Read time variation: {}.{:02}%",
            cv_read / 100,
            cv_read % 100
        );
    }

    if avg_reset > 0 {
        let cv_reset = ((std_dev_reset as u64) * 10000) / avg_reset;
        info!(
            "Reset time variation: {}.{:02}%",
            cv_reset / 100,
            cv_reset % 100
        );
    }
}
