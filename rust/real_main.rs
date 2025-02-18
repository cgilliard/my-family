use prelude::*;

fn test1() {
	let bt = Backtrace::new().unwrap();
	match bt.print() {
		Ok(_) => {}
		Err(e) => {
			println!("Err={}", e);
		}
	}
}

#[no_mangle]
pub extern "C" fn real_main(_argc: i32, _argv: *const *const u8) -> i32 {
	test1();
	0
}
