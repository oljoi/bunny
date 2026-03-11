// SPDX-License-Identifier: GPL-2.0
//
//! bunny
//! you can see it and write to it
//! bun

use kernel::{
  c_str,
  fs::{File, Kiocb},
  global_lock,
  iov::{IovIterDest, IovIterSource},
  miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
  prelude::*,
  time::{delay::fsleep, Delta},
};

module! {
  type: Fluffle,
  name: "bunny",
  authors: ["oljoi"],
  description: "bun",
  license: "GPL",
}

global_lock! {
  unsafe(uninit) static BUNNY_SAY: Mutex<Say> = Say { data: [0; MAX_SAY], len: 0 };
}

const MAX_SAY: usize = 256;
const BUF_SIZE: usize = 1024;

struct Say {
  data: [u8; MAX_SAY],
  len: usize,
}

const MOVE_UP: &[u8] = b"\x1b[7F\x1b[J";

const FRAMES: &[&[u8]] = &[
  b"\
     . (\\_/)\n\
     . (._.)\n\
     . (___)\n",
  b"\
     . (\\_(\\\n\
     . (._.)\n\
     .c(___)\n",
  b"\
     . ()_()\n\
     . (___)\n\
     .(()c())\n",
  b"\
     . /)_/)\n\
     . (._.)\n\
     . (___)>\n",
];

fn push(buf: &mut [u8], pos: &mut usize, src: &[u8]) {
  let end = (*pos + src.len()).min(buf.len());
  let n = end - *pos;
  buf[*pos..*pos + n].copy_from_slice(&src[..n]);
  *pos += n;
}

fn push_byte(buf: &mut [u8], pos: &mut usize, b: u8) {
  if *pos < buf.len() {
    buf[*pos] = b;
    *pos += 1;
  }
}

fn bubble(buf: &mut [u8], pos: &mut usize, msg: &[u8]) {
  if msg.is_empty() {
    for _ in 0..4 {
      push_byte(buf, pos, b'\n');
    }
    return;
  }

  let len = msg.len();

  push_byte(buf, pos, b' ');
  for _ in 0..len + 2 {
    push_byte(buf, pos, b'-');
  }
  push_byte(buf, pos, b'\n');

  push_byte(buf, pos, b'<');
  push_byte(buf, pos, b' ');
  push(buf, pos, msg);
  push_byte(buf, pos, b' ');
  push_byte(buf, pos, b'>');
  push_byte(buf, pos, b'\n');

  push_byte(buf, pos, b' ');
  for _ in 0..len + 2 {
    push_byte(buf, pos, b'-');
  }
  push_byte(buf, pos, b'\n');

  push(buf, pos, b"     \\\n");
}

#[pin_data]
struct Fluffle {
  #[pin]
  _dev: MiscDeviceRegistration<Bunny>,
}

impl kernel::InPlaceModule for Fluffle {
  fn init(_module: &'static ThisModule) -> impl PinInit<Self, Error> {
    pr_info!("bunny is here\n");

    unsafe { BUNNY_SAY.init() };

    let options = MiscDeviceOptions {
      name: c_str!("bunny"),
    };

    try_pin_init!(Self {
      _dev <- MiscDeviceRegistration::register(options),
    })
  }
}

struct Bunny;

#[vtable]
impl MiscDevice for Bunny {
  type Ptr = ();

  fn open(_file: &File, _misc: &MiscDeviceRegistration<Self>) -> Result<()> {
    Ok(())
  }

  fn read_iter(mut kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterDest<'_>) -> Result<usize> {
    let raw = *kiocb.ki_pos_mut();
    let frame_idx = (raw as usize) % FRAMES.len();

    let mut buf = [0u8; BUF_SIZE];
    let mut pos = 0;

    if raw > 0 {
      push(&mut buf, &mut pos, MOVE_UP);
    }

    {
      let msg = BUNNY_SAY.lock();
      bubble(&mut buf, &mut pos, &msg.data[..msg.len]);
    }

    push(&mut buf, &mut pos, FRAMES[frame_idx]);

    let mut local_pos: i64 = 0;
    let written = iov.simple_read_from_buffer(&mut local_pos, &buf[..pos])?;

    *kiocb.ki_pos_mut() = raw + 1;

    fsleep(Delta::from_millis(300));
    Ok(written)
  }

  fn write_iter(mut kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterSource<'_>) -> Result<usize> {
    let len = iov.len().min(MAX_SAY);

    let mut tmp = [0u8; MAX_SAY];
    let copied = iov.copy_from_iter(&mut tmp[..len]);
    if copied == 0 && len > 0 {
      return Err(EFAULT);
    }

    let end = if copied > 0 && tmp[copied - 1] == b'\n' {
      copied - 1
    } else {
      copied
    };

    {
      let mut msg = BUNNY_SAY.lock();
      msg.data[..end].copy_from_slice(&tmp[..end]);
      msg.len = end;
    }

    *kiocb.ki_pos_mut() = 0;
    Ok(copied)
  }
}
