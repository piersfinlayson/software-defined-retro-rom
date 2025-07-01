// sdrr-tester FFI for assembly routines

#[repr(C)]
pub struct ActCsResult {
    pub start_time: u32,
    pub data_in: u32,
    pub end_time: u32,
}

#[link(name = "sdr-tester-asm")]
unsafe extern "C" {
    pub unsafe fn activate_cs_asm(
        cs_gpio_reg: *mut u32,
        cs_mask: u32,
        timer_reg: *const u32,
        gpio_in_reg: *const u32,
        d7_mask: u32,
        result_ptr: *mut ActCsResult
    ) -> u32;  // Return status
}

#[repr(C)]
pub struct DeactCsResult {
    pub start_time: u32,
    pub end_time: u32,
}

#[link(name = "sdr-tester-asm")]
unsafe extern "C" {
    pub unsafe fn deactivate_cs_asm(
        cs_gpio_reg: *mut u32,
        cs_mask: u32,
        timer_reg: *const u32,
        gpio_in_reg: *const u32,
        d7_mask: u32,
        result_ptr: *mut DeactCsResult
    ) -> u32;  // Return status
}