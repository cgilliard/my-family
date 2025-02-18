#[macro_export]
macro_rules! rc {
	($v:expr) => {{
		match Rc::new($v) {
			Ok(v) => match v.clone() {
				Ok(v_clone) => Ok((v, v_clone)),
				Err(e) => Err(e),
			},
			Err(e) => Err(e),
		}
	}};
}

#[macro_export]
macro_rules! lock_pair {
	() => {{
		match lock_box!() {
			Ok(lock1) => match lock1.clone() {
				Ok(lock2) => Ok((lock1, lock2)),
				Err(e) => Err(e),
			},
			Err(e) => Err(e),
		}
	}};
}

#[macro_export]
macro_rules! writeb {
        ($f:expr, $fmt:expr) => {{
            writeb!($f, "{}", $fmt)
        }};
        ($f:expr, $fmt:expr, $($t:expr),*) => {{
            let mut err = err!(Unknown);
            match String::new($fmt) {
                Ok(fmt) => {
                    let mut cur = 0;
                    $(
                        match fmt.findn("{}", cur) {
                            Some(index) => {
                                match fmt.substring( cur, cur + index) {
                                    Ok(s) => {
                                        let s = s.to_str();
                                        match $f.write_str(s, s.len()) {
                                            Ok(_) => {},
                                            Err(e) => err = e,
                                        }
                                        cur += index + 2;
                                    }
                                    Err(e) => err = e,
                                }
                            },
                            None => {
                            },
                        }
                        match $t.format(&mut $f) {
                            Ok(_) => {},
                            Err(e) => err = e,
                        }
                    )*

                    match fmt.substring( cur, fmt.len()) {
                        Ok(s) => {
                            let s = s.to_str();
                            match $f.write_str(s, s.len()) {
                                Ok(_) =>{},
                                Err(e) => err = e,
                            }
                        }
                        Err(e) => err = e,
                    }
                }
                Err(e) => err = e,
            }


            if err.kind == ErrorKind::Unknown {
                Ok(())
            } else {
                Err(err)
            }
        }};
}

#[macro_export]
macro_rules! format {
        ($fmt:expr) => {{
                format!("{}", $fmt)
        }};
        ($fmt:expr, $($t:expr),*) => {{
                let mut formatter = Formatter::new();
                match writeb!(formatter, $fmt, $($t),*) {
                    Ok(_) => String::new(formatter.as_str()),
                    Err(e) => Err(e)
                }
        }};
}

#[macro_export]
macro_rules! exit {
        ($fmt:expr) => {{
                exit!("{}", $fmt);
        }};
        ($fmt:expr,  $($t:expr),*) => {{
                        use ffi::_exit;

                        print!("Panic[@{}:{}]: ", file!(), line!());
                        println!($fmt, $($t),*);
                        match Backtrace::new() {
                                Ok(bt) => { let _ = bt.print(); },
                                Err(_e) => {},
                        }
                        unsafe { _exit(-1); }
                        loop {}
        }};
}

#[cfg(not(test))]
#[macro_export]
macro_rules! panic {
		($fmt:expr) => {{
				exit!("{}", $fmt);
		}};
		($fmt:expr,  $($t:expr),*) => {{
				exit!($fmt, $($t),*);
		}};
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => {{
            #[allow(unused_unsafe)]
            unsafe {
                    let len = $fmt.len();
                    if len > 0 {
                            crate::ffi::write(2, $fmt.as_ptr(), $fmt.len());
                    }
                    crate::ffi::write(2, "\n".as_ptr(), 1);
            }
    }};
    ($fmt:expr, $($t:expr),*) => {{
        match format!($fmt, $($t),*) {
            Ok(line) => {
                    unsafe {
                            let len = line.len();
                            if len > 0 {
                                    crate::ffi::write(2, line.to_str().as_ptr(), line.len());
                            }
                            crate::ffi::write(2, "\n".as_ptr(), 1);
                    }
            },
            Err(_e) => {},
        }
    }};
}

#[macro_export]
macro_rules! print {
    ($fmt:expr) => {{
        unsafe { crate::ffi::write(2, $fmt.as_ptr(), $fmt.len()); }
    }};
    ($fmt:expr, $($t:expr),*) => {{
        match format!($fmt, $($t),*) {
            Ok(line) => {
                unsafe { crate::ffi::write(2, line.to_str().as_ptr(), line.len()); }
            },
            Err(_e) => {},
        }
    }};
}

#[macro_export]
macro_rules! err {
	($kind:expr) => {{
		Error::new($kind, line!(), file!())
	}};
}

#[macro_export]
macro_rules! aadd {
	($a:expr, $v:expr) => {{
		use ffi::atomic_fetch_add_u64;
		unsafe { atomic_fetch_add_u64($a, $v) }
	}};
}

#[macro_export]
macro_rules! asub {
	($a:expr, $v:expr) => {{
		use ffi::atomic_fetch_sub_u64;
		unsafe { atomic_fetch_sub_u64($a, $v) }
	}};
}

#[macro_export]
macro_rules! aload {
	($a:expr) => {{
		use ffi::atomic_load_u64;
		#[allow(unused_unsafe)]
		unsafe {
			atomic_load_u64($a)
		}
	}};
}

#[macro_export]
macro_rules! astore {
	($a:expr, $v:expr) => {{
		use ffi::atomic_store_u64;
		#[allow(unused_unsafe)]
		unsafe {
			atomic_store_u64($a, $v)
		}
	}};
}

#[macro_export]
macro_rules! cas {
	($v:expr, $expect:expr, $desired:expr) => {{
		use ffi::cas_release;
		#[allow(unused_unsafe)]
		unsafe {
			cas_release($v, $expect, $desired)
		}
	}};
}

#[macro_export]
macro_rules! sched_yield {
	() => {{
		use ffi::sched_yield;
		unsafe {
			sched_yield();
		}
	}};
}

#[macro_export]
macro_rules! getmicros {
	() => {{
		use ffi::getmicros;
		unsafe { getmicros() }
	}};
}

#[macro_export]
macro_rules! vec {
                ($($elem:expr),*) => {
                    #[allow(unused_mut)]
                    {
                                let mut vec = Vec::new();
                                let mut err: Error = err!(Unknown);
                                $(
                                        if err.kind == ErrorKind::Unknown {
                                                match vec.push($elem) {
                                                        Ok(_) => {},
                                                        Err(e) => err = e,
                                                }
                                        }
                                )*
                                if err.kind != ErrorKind::Unknown {
                                        Err(err)
                                } else {
                                        Ok(vec)
                                }
                    }
                };
}

#[macro_export]
macro_rules! lock {
	() => {{
		use core::cell::UnsafeCell;
		Lock {
			state: UnsafeCell::new(0),
		}
	}};
}

#[macro_export]
macro_rules! lock_box {
	() => {{
		LockBox::new()
	}};
}
