use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::sync::RwLock;

lazy_static! {
    pub static ref RWLOCK_PATH: RwLock<&'static str> = RwLock::new("");
}

pub fn _create_file_to_write(path: &'static str) {
    let _ = File::create(path);
    let mut rw = RWLOCK_PATH.write().unwrap();
    *rw = path;
}

pub fn _write_to_file(calculated: &f64, result: &str) -> Result<(), io::Error> {
    let rw: &str = &RWLOCK_PATH.read().unwrap();
    if *calculated > 0.0 && *calculated < 9223372036854775807.0 && calculated.fract() == 0.0 {
        let mut output = OpenOptions::new().write(true).append(true).open(rw)?;
        write!(output, "{} = {}\n", *calculated as i64, result)?;
    }
    Ok(())
}
