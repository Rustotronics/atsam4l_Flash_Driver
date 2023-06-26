#![no_std]
#![no_main]

use atsam4lc8c_pac as pac;
// use cortex_m_rt::entry;
use pac::Peripherals;
use panic_halt as _;

pub mod atsam4l;
use atsam4l::FlashWriterEraser;
use core::ptr::write_volatile;
use atsam4l::atsam4lc8c_constants::*;
use pac::HFLASHC;

#[cortex_m_rt::entry]
fn main()->! {
   let addr = 0x0001F400;
   let len = 10;
   let updater = FlashWriterEraser::new();
   updater.hal_flash_erase(addr, len);
   loop {
       //...
   }
}

