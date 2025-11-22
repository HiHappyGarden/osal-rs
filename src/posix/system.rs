static mut START_MAIN_LOOP : bool = false;

pub fn os_version() -> &'static str {
    "POSIX"
}


pub fn start_scheduler() {
    unsafe { START_MAIN_LOOP = true; }
    loop {
        unsafe {
            if !START_MAIN_LOOP {
                break;
            }
        }
    }
}

pub fn stop_scheduler() {
    unsafe { START_MAIN_LOOP = false; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_version() {
        assert_eq!(os_version(), "POSIX");
    }
}