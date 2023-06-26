use atsam4lc8c_pac as pac;
use pac::Peripherals;
use panic_halt as _;

use core::{ptr::write_volatile, convert::TryInto};
use atsam4lc8c_constants::*;
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

// use Flash Command Register (FCMD). 32 bit
// 0 to 5 bits: CMD field. Value: 2. Operation: Erase Page (EP)
// Bits 6 and 7: don't write anything. Not used bits.
// 8 to 15: PAGEN field: it is used to address a page or fuse bit for certain operations.  PAGEN field 
    // is automatically updated every time the page buffer is written to
// 16 to 23: PAGEN field: same as above.
// Total page no. :1024 (0 to 1023). In Hex = 0x400. In Binary, 1024 = 0100 0000 0000.
// which means, to write 1023 (Bin= 11 0000 0000) in binary we need only 10 bits.
// 24 to 31: KEY: write protection key. Value to be written: 0xA5 for the CMD to be enabled. If
    // any different value is written, then write operation is not performed and no action will be taken.




/* Flash must be erased to program it. Use Erase all or Erase Page command */

// Erase the page to be programmed

/* To write data use "Page buffer"
  Can lock region after coding. Locking 1 region will lock all the pages under that region
 Writing of 8-bit and 16-bit data to the page buffer is not allowed and may lead to unpredictable data corruption
*/ 

// Reset the page buffer with the Clear Page Buffer command

// Fill the page buffer with the desired content

/* Programming starts as soon as the programming key and the programming command are
written to the Flash Command Register. The PAGEN field in the Flash Command Register
(FCMD) must contain the address of the page to write. PAGEN is automatically updated
when writing to the page buffer, but can also be written to directly. The FRDY bit in the Flash
Status Register (FSR) is automatically cleared when the page write operation starts. */

// Address of the page to be written should be in "PAGEN" field in the Flash Command Register (FCMD) 

/* PAGEN is automatically updated when writing to the page buffer, but can also be written to directly. 
 The FRDY bit in the Flash
Status Register (FSR) is automatically cleared when the page write operation starts. */

// TO WRITE:
// The flash memory has a write and erase granularity of one page; data is written and erased in chunks of one page. 
// To program a page. First write to page buffer
// To copy contents from Page buffer to desired page in flash, use "Write page" command
// Internally, the flash memory stores data in 64-bit doublewords
// 
 /* The page buffer is not automatically reset after a page write. The user should do this manually
by issuing the Clear Page Buffer flash command. This can be done after a page write, or before
the page buffer is loaded with data to be stored to the flash page */


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
    // fn hal_flash_write(&self, address: usize, data: *const u8, len: usize) {
    //     let address = address as u32;
    //     let len = len as u32;
    //     let mut idx = 0u32;
    //     let mut src = data as *mut u32;
    //     let mut dst = address as *mut u32;
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
        let ending_page = ((addr + len)/512) as u32;


            // CMD, PAGEN and KEY are the fields of FCMD register
        for addr in starting_page..ending_page {
        //    println!("Page number is:{}", addr);
            self.nvm.fcmd.write(|s|
                unsafe { s.cmd().bits(0x02) });
            self.nvm.fcmd.write(|s|
                unsafe { s.pagen().bits(addr.try_into().unwrap()) });
            self.nvm.fcmd.write(|s|
                unsafe { s.key().bits(0xA5) });


            loop{
                let fsr_status = self.nvm.fsr.read().frdy().bit();
                if fsr_status == true {
                   // println!("Erasing successful");
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
