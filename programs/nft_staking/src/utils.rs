use anchor_lang::{
  prelude::*, 
  solana_program::{
    clock,
    keccak::{hash, Hash}
  }
};

pub fn get_now_timestamp() -> u64 {
  clock::Clock::get()
    .unwrap()
    .unix_timestamp
    .try_into()
    .unwrap()
}

pub fn get_random() -> u32 {
  let slot = clock::Clock::get().unwrap().slot;
  let hash = hash(&slot.to_be_bytes());
  let buf: [u8; 32] = Hash::to_bytes(hash);
  let slice: [u8; 4] = [buf[10], buf[12], buf[8], buf[16]];
  u32::from_be_bytes(slice)  
}