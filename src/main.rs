#![no_std]
#![no_main]

use defmt_rtt as _;
use atsam4lc8c_pac as pac;
use pac::{Peripherals, aesa::databufptr, smap::length};
use panic_halt as _;
use defmt::println;



pub mod atsam4l;
use atsam4l::FlashWriterEraser;
use core::ptr::write_volatile;
use atsam4l::atsam4lc8c_constants::*;
use pac::HFLASHC;

#[cortex_m_rt::entry]
fn main()->! {
   let addr = 0x00048300;
   let len = 39;
   let data: [u8; 513] = [0xA4; 513];
   // let mut data_length = data.len();
   let raw_ptr = data.as_ptr();
   let mut updater = FlashWriterEraser::new();
   // updater.hal_flash_erase(addr, len);
   updater.hal_flash_write(0x00048300, data.as_ptr(), len);
   // updater.write_nvm_words(0x00043800, &data, len);
   // updater.write_nvm_word(0x00048300, &data, len);
   defmt::println!("Writing finished");
   loop {
       //...
   }
}

