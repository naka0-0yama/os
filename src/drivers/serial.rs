use crate::arch::io::{inb, outb};
use core::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SerialWriter {
    pub port: u16,
    pub serial_lock: AtomicBool,
}

pub static COM1: SerialWriter = SerialWriter::new(0x3F8);

impl SerialWriter {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            serial_lock: AtomicBool::new(false),
        }
    }

    pub fn init(&self) {
        unsafe {
            outb(self.port + 1, 0x00);
            outb(self.port + 3, 0x80);

            outb(self.port + 0, 0x03);
            outb(self.port + 1, 0x00);

            outb(self.port + 3, 0x03);
            outb(self.port + 2, 0xC7);
            outb(self.port + 4, 0x0B);
        }
    }

    fn lock(&self) -> u64 {
        let flags: u64;

        unsafe {
            core::arch::asm!("pushfq", "pop {}", out(reg) flags, options(nomem, preserves_flags));
            core::arch::asm!("cli", options(nomem, nostack));
        }

        while self
            .serial_lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        flags
    }

    fn unlock(&self, saved: u64) {
        self.serial_lock.store(false, Ordering::Release);

        if saved & (1 << 9) != 0 {
            unsafe {
                core::arch::asm!("sti", options(nomem, nostack));
            }
        }
    }

    fn emit(&self, byte: u8) {
        unsafe {
            while inb(self.port + 5) & 0x20 == 0 {
                core::hint::spin_loop();
            }

            outb(self.port, byte);
        }
    }

    pub fn putc(&self, byte: u8) {
        let saved = self.lock();
        self.emit(byte);
        self.unlock(saved);
    }

    fn write_bytes(&self, bytes: &[u8]) {
        let saved = self.lock();

        for &byte in bytes {
            self.emit(byte);
        }

        self.unlock(saved);
    }

    pub fn print(&self, text: &str) {
        self.write_bytes(text.as_bytes());
    }

    pub fn println(&self, text: &str) {
        let saved = self.lock();

        for byte in text.bytes() {
            self.emit(byte);
        }

        self.emit(b'\n');
        self.unlock(saved);
    }

    pub fn printhexu32(&self, value: u32) {
        let hex = b"0123456789ABCDEF";
        let saved = self.lock();

        for i in (0..8).rev() {
            let digit = ((value >> (i * 4)) & 0xF) as usize;
            self.emit(hex[digit]);
        }

        self.unlock(saved);
    }

    pub fn printhexu64(&self, value: u64) {
        let hex = b"0123456789ABCDEF";
        let saved = self.lock();

        for i in (0..16).rev() {
            let digit = ((value >> (i * 4)) & 0xF) as usize;
            self.emit(hex[digit]);
        }

        self.unlock(saved);
    }

    pub fn printu64(&self, mut value: u64) {
        let saved = self.lock();

        if value == 0 {
            self.emit(b'0');
            self.unlock(saved);
            return;
        }

        let mut buffer = [0u8; 20];
        let mut index = 0;

        while value > 0 {
            buffer[index] = b'0' + (value % 10) as u8;
            value /= 10;
            index += 1;
        }

        while index > 0 {
            index -= 1;
            self.emit(buffer[index]);
        }

        self.unlock(saved);
    }

    pub fn write_fmt(&self, args: fmt::Arguments) {
        struct Adapter<'a>(&'a SerialWriter);

        impl fmt::Write for Adapter<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.0.write_bytes(s.as_bytes());
                Ok(())
            }
        }

        let mut adapter = Adapter(self);
        let _ = fmt::write(&mut adapter, args);
    }

    fn emergency_putc(&self, byte: u8) {
        let mut timeout = 1_000_000usize;

        unsafe {
            while inb(self.port + 5) & 0x20 == 0 {
                if timeout == 0 {
                    return;
                }

                timeout -= 1;
                core::hint::spin_loop();
            }

            outb(self.port, byte);
        }
    }

    pub fn emergency_write_fmt(&self, args: fmt::Arguments) {
        struct EmergencyWriter<'a>(&'a SerialWriter);

        impl fmt::Write for EmergencyWriter<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                for byte in s.bytes() {
                    self.0.emergency_putc(byte);
                }

                Ok(())
            }
        }

        let mut writer = EmergencyWriter(self);
        let _ = fmt::write(&mut writer, args);
    }
}

#[macro_export]
macro_rules! serialprint {
    ($($arg:tt)*) => {
        $crate::drivers::serial::COM1.write_fmt(
            format_args!($($arg)*)
        );
    };
}

#[macro_export]
macro_rules! serialprintln {
    () => {
        $crate::serialprint!("\n");
    };

    ($($arg:tt)*) => {
        $crate::serialprint!("{}\n", format_args!($($arg)*));
    };
}
