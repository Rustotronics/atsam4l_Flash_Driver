use atsam4lc8c_pac as pac;
use pac::{smap::addr, Peripherals};
use panic_halt as _;

use atsam4lc8c_constants::*;
use core::{convert::TryInto, ptr::write_volatile};
use pac::HFLASHC;

#[rustfmt::skip]
// ATSAM4LC8CA has page size = 512 Bytes
// Flash size = 512KB
// No. of pages = 1024
pub mod atsam4lc8c_constants {
    pub const FLASH_PAGE_SIZE : u32 = 512;   // 1 page size = 512 Bytes   
    pub const STACK_LOW       : u32 = 0x2000_0000;
    pub const STACK_UP        : u32 = 0x2002_0000;
    pub const RB_HDR_SIZE     : u32 = 0x100;
    pub const BASE_ADDR       : u32 = 0x08020000;   //  sector 5 starting address
    pub const VTR_TABLE_SIZE  : u32 = 0x100;
    pub const FW_RESET_VTR    : u32 = BASE_ADDR + RB_HDR_SIZE + VTR_TABLE_SIZE + 0x99;
    pub const UNLOCKKEY1      : u32 = 0x45670123;
    pub const UNLOCKKEY2      : u32 = 0xCDEF89AB;
    pub const PSIZE_X8        : u8  = 0b00;
    pub const PSIZE_X16       : u8  = 0b01;
    pub const PSIZE_X32       : u8  = 0b10;
    pub const PSIZE_X64       : u8  = 0b11;
}

// TO ERASE the FLASH

// use Flash Command Register (FCMD).
// FCMD has 3 fields: KEY, PAGEN, CMD. Refer Datasheet Chapter 14 to know more

pub struct FlashWriterEraser {
    pub nvm: HFLASHC,
}

impl FlashWriterEraser {
    pub fn new() -> Self {
        FlashWriterEraser {
            nvm: Peripherals::take().unwrap().HFLASHC,
        }
    }
}

impl FlashWriterEraser {
    /// This method is to write data on flash
    ///
    /// Method arguments:
    /// -   address: It holds the address of flash where data has to be written
    /// -   data: u8 pointer holding the holding data.
    /// -   len :  number of bytes
    ///
    /// Returns:
    /// -  NONE

    pub fn hal_flash_write(&self, address: usize, data: *const u8, len: usize) {
        let address = address as u32;
        let len = len as u32;

        let starting_page = (address / 512) as u32;
        let ending_page = ((address + len) / 512) as u32;
        let mut idx = 0u32;
        let mut src = data as *mut u32;
        let mut dst = address as *mut u32;
        let mut pg_num = address/512;

        defmt::println!("Starting page calculated");

        // If len < 1 page, then this block is used to earse the page
        if starting_page == ending_page{
            self.nvm.fcmd.write(|s| unsafe {
                s.key().bits(0xA5)
                 .pagen().bits(pg_num.try_into().unwrap())
                 .cmd().bits(0x02)
            });
            defmt::println!("am I erasing the page?");
        }
        loop {
            let fsr_status = self.nvm.fsr.read().frdy().bit();
            if fsr_status == true {
                // defmt::println!("Erase successful");
                break;
            }
        } 
        for addr in starting_page..ending_page {
            defmt::println!("addr value in for loop: {}", addr);
            // Perform erase operation on pages to be written
            self.nvm.fcmd.write(|s| unsafe {
                s.key().bits(0xA5)
                 .pagen().bits(addr.try_into().unwrap())
                 .cmd().bits(0x02)
            });
            // defmt::println!("What's the addr value here {}", addr);
            loop {
                let fsr_status = self.nvm.fsr.read().frdy().bit();
                if fsr_status == true {
                    // defmt::println!("Erase successful");
                    break;
                }
            }          
        }  

        // Before writing to FCMD, first write to "Page Buffer"
        // Use "write page" command
        while idx < len {
            // let data_ptr = 

            // Check if the following holds true and do a full word write i.e. 4-byte write
            // - if `len - idx` is greater than 3 (i.e. 4 bytes).
            // - if the address is aligned on a word (i.e. 4-byte) boundary.
            // - if the data_ptr is aligned on a word (i.e. 4-byte) boundary.
            // if (len - idx > 3)
            //     && ((((address + idx) & 0x03) == 0) && ((data_ptr + idx) & 0x03) == 0)
            // {
                // To begin write operation, Clear page buffer register
                self.nvm.fcmd.write(|s| unsafe {
                    s.key().bits(0xA5)
                // pagen field is not used
                        .cmd().bits(0x03)
                });

                loop {
                    let fsr_status = self.nvm.fsr.read().frdy().bit();
                    if fsr_status == true {
                        // defmt::println!("Erase successful");
                        break;
                    }
                } 

                // Write to Page Buffer by directly writing to flash memory
                unsafe {
                    *dst = *src; // 4-byte write (For lower part of doubleword)
                };

                src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
                dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                unsafe {
                    *dst = *src; // 4-byte write (For higher part of doubleword)
                };

                src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
                dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                
                // Flash write command
                self.nvm.fcmd.write(|s| unsafe {
                    s.key().bits(0xA5)
                    .pagen().bits(pg_num.try_into().unwrap())
                    .cmd().bits(0x01)
                });
                // defmt::println!("Addr value: {}", addr);

                loop {
                    let fsr_status = self.nvm.fsr.read().frdy().bit();
                    if fsr_status == true {
                        // defmt::println!("Erase successful");
                        break;
                    }
                }
                // increment index value
                idx += 8;

                if (idx%512)==0 {
                    pg_num+=1;
                }
            // } else {
                // else do a single byte write i.e. 1-byte write with padding for rest of the bytes
            // }
        }
}
    //     //Unlock the FLASH
    //     self.hal_flash_unlock();
    //     while idx < len {
    //         let data_ptr = (data as *const u32) as u32;
    //         //checking if the len is more than 4 bytes to compute a 4 byte write on flash
    //         if (len - idx > 3) {
    //             // Enable FLASH Page writes
    //             self.nvm.cr.modify(|_, w| unsafe {
    //                 w.psize()
    //                     .bits(PSIZE_X32)
    //                     // no sector erase
    //                     .ser()
    //                     .clear_bit()
    //                     // programming
    //                     .pg()
    //                     .set_bit()
    //             });
    //             while self.nvm.sr.read().bsy().bit() {}
    //             unsafe {
    //                 // *dst = data; // 4-byte write
    //                 write_volatile(dst, *src);
    //             };

    //             src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
    //             dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
    //             idx += 4;
    //         } else {
    //             // else do a single byte write i.e. 1-byte write
    //             let mut val = 0u32;
    //             let val_bytes = ((&mut val) as *mut u32) as *mut u8;
    //             let offset = (address + idx) - (((address + idx) >> 2) << 2); // offset from nearest word aligned address
    //             dst = ((dst as u32) - offset) as *mut u32; // subtract offset from dst addr
    //             unsafe {
    //                 val = *dst; // assign current val at dst to val
    //                             // store data byte at idx to `val`. `val_bytes` is a byte-pointer to val.
    //                 *val_bytes.add(offset as usize) = *data.add(idx as usize);
    //             }
    //             // Enable FLASH Page writes
    //             self.nvm.cr.modify(|_, w| unsafe {
    //                 w.psize()
    //                     .bits(PSIZE_X32)
    //                     // no sector erase
    //                     .ser()
    //                     .clear_bit()
    //                     // programming
    //                     .pg()
    //                     .set_bit()
    //             });
    //             while self.nvm.sr.read().bsy().bit() {}
    //             unsafe {
    //                 *dst = val; // Technically this is a 1-byte write ONLY
    //                             // but only full 32-bit words can be written to Flash using the NVMC interface
    //             };
    //             src = ((src as u32) + 1) as *mut u32; // increment pointer by 1
    //             dst = ((dst as u32) + 1) as *mut u32; // increment pointer by 1
    //             idx += 1;
    //         }
    //     }
    //     //Lock the FLASH
    //     self.hal_flash_lock();
    // }

    /// This method is used to erase data on flash
    ///
    /// First, we need to find the page number and specify the page number in PAGEN field of FCMD register
    ///
    /// Method arguments:
    /// -   addr: Address where data has to be erased
    /// -   len :  number of bytes to be erased
    ///
    /// Returns:
    /// -  NONE

    pub fn hal_flash_erase(&self, addr: usize, len: usize) {
        // 1 page size = 512. address / 512 (Integer part of the result) = page number.
        // Feed the page number to FCMD register in PAGEN field.

        let starting_page = (addr / 512) as u32;
        let ending_page = ((addr + len) / 512) as u32;

        for addr in starting_page..ending_page {
            // defmt::println!("addr value in for loop: {}", addr);
            self.nvm.fcmd.write(|s| unsafe {
                s.key().bits(0xA5)
                 .pagen().bits(addr.try_into().unwrap())
                 .cmd().bits(0x02)
            });

            // defmt::println!("Erasing pages successful");
            // defmt::println!("Bits value is: {}", self.nvm.fsr.read().bits());
            // defmt::println!("Locke value is: {}", self.nvm.fsr.read().locke().bit());

            loop {
                let fsr_status = self.nvm.fsr.read().frdy().bit();
                if fsr_status == true {
                    // defmt::println!("Erase successful");
                    break;
                }
            }
        }
    }
}

//     /// This method is used to lock the flash
//     ///
//     /// Once the flash is locked no operation on flash can be perfomed.
//     /// Method arguments:
//     /// -   NONE
//     /// Returns:
//     /// -  NONE
//     fn hal_flash_lock(&self) {
//         self.nvm.cr.modify(|_, w| w.lock().set_bit());
//     }
//     /// This method is used to unlock the flash
//     ///
//     /// Flash has to be unlocked to do any operation on it.
//     /// Method arguments:
//     /// -   NONE
//     /// Returns:
//     /// -  NONE
//     fn hal_flash_unlock(&self) {
//         self.nvm.keyr.write(|w| unsafe { w.key().bits(UNLOCKKEY1) });
//         self.nvm.keyr.write(|w| unsafe { w.key().bits(UNLOCKKEY2) });
//     }
//     fn hal_init() {}

// pub fn preboot() {}

// struct RefinedUsize<const MIN: u32, const MAX: u32, const VAL: u32>(u32);

// impl<const MIN: u32, const MAX: u32, const VAL: u32> RefinedUsize<MIN, MAX, VAL> {
//     /// This method is used to check the address bound of stack pointer
//     ///
//     /// Method arguments:
//     /// -   i : starting address of stack
//     /// Returns:
//     /// -  It returns u32 address of stack pointer
//     pub fn bounded_int(i: u32) -> Self {
//         assert!(i >= MIN && i <= MAX);
//         RefinedUsize(i)
//     }
//     /// This method is used to check the address of reset pointer
//     ///
//     /// Method arguments:
//     /// -   i : starting address of reset
//     /// Returns:
//     /// -  It returns u32 address of reset pointer
//     pub fn single_valued_int(i: u32) -> Self {
//         assert!(i == VAL);
//         RefinedUsize(i)
//     }
// }

// /// This method is used to boot the firmware from a particular address
//     ///
//     /// Method arguments:
//     /// -   fw_base_address  : address of the firmware
//     /// Returns:
//     /// -  NONE
// #[rustfmt::skip]
// pub fn boot_from(fw_base_address: usize) -> ! {
//        let address = fw_base_address as u32;
//        let scb = hal::pac::SCB::ptr();
//        unsafe {
//        let sp = RefinedUsize::<STACK_LOW, STACK_UP, 0>::bounded_int(
//         *(fw_base_address as *const u32)).0;
//        let rv = RefinedUsize::<0, 0, FW_RESET_VTR>::single_valued_int(
//         *((fw_base_address + 4) as *const u32)).0;
//        let jump_vector = core::mem::transmute::<usize, extern "C" fn() -> !>(rv as usize);
//        (*scb).vtor.write(address);
//        cortex_m::register::msp::write(sp);
//        jump_vector();

//        }
//        loop{}
// }
