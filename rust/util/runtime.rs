use prelude::*;

type Task<T> = Box<dyn FnMut() -> T>;

pub struct RuntimeConfig {
	pub min_threads: u64,
	pub max_threads: u64,
}

pub struct Handle<T> {
	channel: Receiver<T>,
	is_complete: Rc<bool>,
}

struct JhEntry {
	id: u64,
	jh: Option<JoinHandle>,
}

struct State {
	total_workers: u64,
	waiting_workers: u64,
	halt: bool,
	jhs: Hashtable<JhEntry>,
}

enum Message<T> {
	Task((Task<T>, Sender<T>, Rc<bool>)),
	Halt,
}

pub struct Runtime<T> {
	config: RuntimeConfig,
	send: Sender<Message<T>>,
	recv: Receiver<Message<T>>,
	state: Rc<State>,
	lock: LockBox,
	counter: u64,
}

impl PartialEq for JhEntry {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Hash for JhEntry {
	fn hash(&self) -> usize {
		murmur3_32_of_u64(self.id, get_murmur_seed()) as usize
	}
}

impl Default for RuntimeConfig {
	fn default() -> Self {
		Self {
			min_threads: 4,
			max_threads: 8,
		}
	}
}

impl<T> Drop for Runtime<T> {
	fn drop(&mut self) {
		let _ = self.stop();
	}
}

impl<T> Handle<T> {
	pub fn block_on(&self) -> T {
		self.channel.recv()
	}

	pub fn is_complete(&self) -> bool {
		*self.is_complete
	}
}

impl<T> Runtime<T> {
	pub fn new(config: RuntimeConfig) -> Result<Self, Error> {
		let (send, recv) = match channel() {
			Ok((send, recv)) => (send, recv),
			Err(e) => return Err(e),
		};
		let jhs = match Hashtable::new(config.max_threads as usize * 2) {
			Ok(jhs) => jhs,
			Err(e) => return Err(e),
		};
		let state = match Rc::new(State {
			waiting_workers: 0,
			total_workers: config.min_threads,
			halt: false,
			jhs,
		}) {
			Ok(state) => state,
			Err(e) => return Err(e),
		};
		let lock = match lock_box!() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};

		Ok(Self {
			config,
			send,
			recv,
			state,
			lock,
			counter: 0,
		})
	}

	pub fn start(&mut self) -> Result<(), Error> {
		// SAFETY: clone always succeeds on LockBox
		let lock = self.lock.clone().unwrap();
		{
			let _l = lock.read();
			if self.state.halt {
				return Err(err!(NotInitialized));
			}
		}
		for _i in 0..self.config.min_threads {
			match self.thread(self.config.min_threads, self.config.max_threads) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		{
			let _l = self.lock.write();
			if self.state.halt {
				return Err(err!(NotInitialized));
			}
			self.state.halt = true;
		}
		for _i in 0..self.config.max_threads {
			match self.send.send(Message::Halt) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		for mut ent in &self.state.jhs {
			match &mut (*ent).value.jh {
				Some(entry) => {
					let _ = entry.join();
				}
				None => {}
			}
			ent.release();
		}

		Ok(())
	}

	pub fn execute<F>(&mut self, task: F) -> Result<Handle<T>, Error>
	where
		F: FnMut() -> T + 'static,
	{
		{
			let _l = self.lock.read();
			if self.state.halt {
				return Err(err!(NotInitialized));
			}
		}
		let (send, recv) = match channel() {
			Ok((send, recv)) => (send, recv),
			Err(e) => return Err(e),
		};
		let rc = match Rc::new(false) {
			Ok(rc) => rc,
			Err(e) => return Err(e),
		};
		// SAFETY: rc.clone always succeeds
		let rc_clone = rc.clone().unwrap();
		let task = match Box::new(task) {
			Ok(task) => task,
			Err(e) => return Err(e),
		};
		let msg = Message::Task((task, send, rc));
		match self.send.send(msg) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		Ok(Handle {
			channel: recv,
			is_complete: rc_clone,
		})
	}

	#[cfg(test)]
	fn cur_threads(&self) -> u64 {
		let _l = self.lock.read();
		self.state.total_workers
	}

	#[cfg(test)]
	fn idle_threads(&self) -> u64 {
		let _l = self.lock.read();
		self.state.waiting_workers
	}

	fn thread(&mut self, min: u64, max: u64) -> Result<(), Error> {
		let id = aadd!(&mut self.counter, 1);
		// SAFETY: unwraps are ok because they are clone for rc, lock, and channels
		// which do not fail
		let recv = self.recv.clone().unwrap();
		let mut state = self.state.clone().unwrap();
		let mut state_clone = state.clone().unwrap();
		let lock = self.lock.clone().unwrap();
		let lock_clone = lock.clone().unwrap();

		let jh = match spawnj(move || loop {
			{
				let _l = lock.write();
				if state.halt {
					state.total_workers -= 1;
					break;
				} else {
					state.waiting_workers += 1;
					if state.waiting_workers > min {
						state.total_workers -= 1;
						state.waiting_workers -= 1;
						let jhent = state.jhs.remove(&JhEntry { id, jh: None }).unwrap();
						jhent.release();
						break;
					}
				}
			}
			match recv.recv() {
				Message::Task(mut t) => {
					{
						let mut do_spawn = false;
						{
							let _l = lock.write();
							state.waiting_workers -= 1;
							if state.waiting_workers == 0
								&& state.total_workers < max
								&& !state.halt
							{
								state.total_workers += 1;
								do_spawn = true;
							}
						}
						if do_spawn {
							match self.thread(min, max) {
								Ok(_) => {}
								Err(e) => {
									println!("WARN: Could not start additional thread: ", e)
								}
							}
						}
					}
					let res = t.0();
					*t.2 = true;
					match t.1.send(res) {
						Ok(_) => {}
						Err(e) => {
							println!("WARN: could not send result: ", e);
						}
					}
				}
				Message::Halt => {}
			}
		}) {
			Ok(jh) => jh,
			Err(e) => {
				return Err(e);
			}
		};

		let _l = lock_clone.write();
		let jhent = JhEntry { jh: Some(jh), id };
		let ptr = match Ptr::alloc(Node::new(jhent)) {
			Ok(ptr) => ptr,
			Err(e) => return Err(e),
		};
		state_clone.jhs.insert(ptr);

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use ffi::getalloccount;
	#[test]
	fn test_runtime1() {
		let initial = unsafe { getalloccount() };
		{
			let mut x = Runtime::new(RuntimeConfig::default()).unwrap();
			assert!(x.start().is_ok());
			let (send1, recv1) = channel().unwrap();
			let (send2, recv2) = channel().unwrap();
			let handle1 = x
				.execute(move || -> i32 {
					assert_eq!(recv1.recv(), 8);
					7
				})
				.unwrap();

			assert!(!handle1.is_complete());
			send1.send(8).unwrap();

			assert_eq!(handle1.block_on(), 7);
			assert!(handle1.is_complete());

			let handle2 = x
				.execute(move || -> i32 {
					send2.send(9).unwrap();
					6
				})
				.unwrap();

			assert_eq!(recv2.recv(), 9);
			assert_eq!(handle2.block_on(), 6);
			assert!(handle2.is_complete());

			assert!(x.stop().is_ok());
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_runtime2() {
		let config = RuntimeConfig {
			min_threads: 2,
			max_threads: 3,
		};
		let mut x: Runtime<()> = Runtime::new(config).unwrap();
		assert!(x.start().is_ok());
		let (send1, recv1) = channel().unwrap();
		let (send2, recv2) = channel().unwrap();
		let (senda1, recva1) = channel().unwrap();
		let (senda2, recva2) = channel().unwrap();

		let h1 = x
			.execute(move || {
				send1.send(()).unwrap();
				recva1.recv();
			})
			.unwrap();

		let h2 = x
			.execute(move || {
				send2.send(()).unwrap();
				recva2.recv();
			})
			.unwrap();

		recv1.recv();
		recv2.recv();

		while x.idle_threads() != 1 {}
		assert_eq!(x.idle_threads(), 1);
		assert_eq!(x.cur_threads(), 3);

		assert!(senda1.send(()).is_ok());
		assert!(senda2.send(()).is_ok());

		assert_eq!(h1.block_on(), ());
		assert_eq!(h2.block_on(), ());

		while x.cur_threads() != 2 {}
		assert_eq!(x.cur_threads(), 2);
		assert_eq!(x.idle_threads(), 2);

		assert!(x.stop().is_ok());
	}

	#[test]
	fn test_thread_pool_size() {
		let initial = unsafe { getalloccount() };
		{
			let mut r = Runtime::new(RuntimeConfig {
				min_threads: 2,
				max_threads: 4,
			})
			.unwrap();
			r.start().unwrap();

			while r.idle_threads() != 2 {}

			let (senda1, recva1) = channel().unwrap();
			let (sendb1, recvb1) = channel().unwrap();
			let (sendc1, recvc1) = channel().unwrap();

			let x1 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva1.recv(), 1);
					sendb1.send(1).unwrap();
					assert_eq!(recvc1.recv(), 1);
					Ok(1)
				})
				.unwrap();

			let (senda2, recva2) = channel().unwrap();
			let (sendb2, recvb2) = channel().unwrap();
			let (sendc2, recvc2) = channel().unwrap();

			let x2 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva2.recv(), 2);
					sendb2.send(2).unwrap();
					assert_eq!(recvc2.recv(), 2);
					Ok(2)
				})
				.unwrap();

			senda1.send(1).unwrap();
			senda2.send(2).unwrap();

			assert_eq!(recvb1.recv(), 1);
			assert_eq!(recvb2.recv(), 2);

			// we know there should be three threads spawned at this point because at least one
			// waiting worker is maintained.
			assert_eq!(r.cur_threads(), 3);

			sendc1.send(1).unwrap();
			sendc2.send(2).unwrap();

			assert_eq!(x1.block_on().unwrap(), 1);
			assert_eq!(x2.block_on().unwrap(), 2);

			while r.cur_threads() != 2 {}

			// The other two threads have exited so we should be back down to our min
			assert_eq!(r.cur_threads(), 2);

			// now start up 5 threads (we'll hit our limit of 4)
			let (senda1, recva1) = channel().unwrap();
			let (sendb1, recvb1) = channel().unwrap();
			let (sendc1, recvc1) = channel().unwrap();

			let x1 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva1.recv(), 1);
					sendb1.send(1).unwrap();
					assert_eq!(recvc1.recv(), 1);
					Ok(1)
				})
				.unwrap();

			let (senda2, recva2) = channel().unwrap();
			let (sendb2, recvb2) = channel().unwrap();
			let (sendc2, recvc2) = channel().unwrap();

			let x2 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva2.recv(), 2);
					sendb2.send(2).unwrap();
					assert_eq!(recvc2.recv(), 2);
					Ok(2)
				})
				.unwrap();

			let (senda3, recva3) = channel().unwrap();
			let (sendb3, recvb3) = channel().unwrap();
			let (sendc3, recvc3) = channel().unwrap();

			let x3 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva3.recv(), 3);
					sendb3.send(3).unwrap();
					assert_eq!(recvc3.recv(), 3);
					Ok(3)
				})
				.unwrap();

			let (senda4, recva4) = channel().unwrap();
			let (sendb4, recvb4) = channel().unwrap();
			let (sendc4, recvc4) = channel().unwrap();

			let x4 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva4.recv(), 4);
					sendb4.send(4).unwrap();
					assert_eq!(recvc4.recv(), 4);
					Ok(4)
				})
				.unwrap();

			let (senda5, recva5) = channel().unwrap();
			let (sendb5, recvb5) = channel().unwrap();
			let (sendc5, recvc5) = channel().unwrap();

			let x5 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva5.recv(), 5);
					sendb5.send(5).unwrap();
					assert_eq!(recvc5.recv(), 5);
					Ok(5)
				})
				.unwrap();

			senda1.send(1).unwrap();
			senda2.send(2).unwrap();
			senda3.send(3).unwrap();
			senda4.send(4).unwrap();

			assert_eq!(recvb1.recv(), 1);
			assert_eq!(recvb2.recv(), 2);
			assert_eq!(recvb3.recv(), 3);
			assert_eq!(recvb4.recv(), 4);

			// we are now at our max threads (4) there would have been a 5th, but we hit the
			// max.
			assert_eq!(r.cur_threads(), 4);

			// send messages to release all threads
			senda5.send(5).unwrap();
			sendc1.send(1).unwrap();
			sendc2.send(2).unwrap();
			sendc3.send(3).unwrap();
			sendc4.send(4).unwrap();
			sendc5.send(5).unwrap();

			// thread 5 can now complete
			assert_eq!(recvb5.recv(), 5);

			while r.cur_threads() != 2 {}

			// After things settle down we should return to our min thread level of 2
			assert_eq!(r.cur_threads(), 2);

			assert_eq!(x1.block_on().unwrap(), 1);
			assert_eq!(x2.block_on().unwrap(), 2);
			assert_eq!(x3.block_on().unwrap(), 3);
			assert_eq!(x4.block_on().unwrap(), 4);
			assert_eq!(x5.block_on().unwrap(), 5);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
