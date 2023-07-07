#![no_std]
#![no_main]

use defmt_rtt as _;
use atsam4lc8c_pac as pac;
use pac::{Peripherals, aesa::databufptr};
use panic_halt as _;
use defmt::println;



pub mod atsam4l;
use atsam4l::FlashWriterEraser;
use core::ptr::write_volatile;
use atsam4l::atsam4lc8c_constants::*;
use pac::HFLASHC;

#[cortex_m_rt::entry]
fn main()->! {
   let addr = 0x00043800;
   let len = 500;
   let data: [u8; 500] = [0xA5; 500];
   let raw_ptr = data.as_ptr();
   let updater = FlashWriterEraser::new();
   // updater.hal_flash_erase(addr, len);
   updater.hal_flash_write(0x00043800, data.as_ptr(), 500);
   defmt::println!("Writing finished");
   loop {
       //...
   }
}

