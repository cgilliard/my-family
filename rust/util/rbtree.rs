use core::clone::Clone;
use core::ops::FnMut;
use core::ptr::null_mut;
use prelude::*;

pub struct RbNodePair<V: Ord> {
	pub cur: Ptr<RbTreeNode<V>>,
	pub parent: Ptr<RbTreeNode<V>>,
	pub is_right: bool,
}

type RbTreeSearch<V> = dyn FnMut(Ptr<RbTreeNode<V>>, Ptr<RbTreeNode<V>>) -> RbNodePair<V>;

pub struct RbTreeNode<V: Ord> {
	pub parent: Ptr<RbTreeNode<V>>,
	pub right: Ptr<RbTreeNode<V>>,
	pub left: Ptr<RbTreeNode<V>>,
	pub value: V,
}

enum Color {
	Black,
	Red,
}

impl<V: Ord> Display for RbTreeNode<V> {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		writeb!(
			*f,
			"Node: parent={},left={},right={},color={},bitcolor={}",
			self.parent,
			self.left,
			self.right,
			if self.is_red() { "red" } else { "black" },
			if self.parent.get_bit() {
				"red"
			} else {
				"black"
			}
		)
	}
}

impl<V: Ord> RbTreeNode<V> {
	pub fn new(value: V) -> Self {
		Self {
			parent: Ptr::new_bit_set(null_mut()),
			right: Ptr::null(),
			left: Ptr::null(),
			value,
		}
	}

	fn set_color(&mut self, color: Color) {
		match color {
			Color::Black => {
				self.parent.set_bit(false);
			}
			Color::Red => {
				self.parent.set_bit(true);
			}
		}
	}

	fn is_root(&self) -> bool {
		self.parent.is_null()
	}

	fn is_red(&self) -> bool {
		self.parent.get_bit()
	}

	fn is_black(&self) -> bool {
		!self.is_red()
	}

	fn set_parent(&mut self, parent: Ptr<Self>) {
		match self.is_black() {
			true => {
				self.parent = parent;
				self.parent.set_bit(false);
			}
			false => {
				self.parent = parent;
				self.parent.set_bit(true);
			}
		}
	}
}

pub struct RbTree<V: Ord> {
	root: Ptr<RbTreeNode<V>>,
}

impl<V: Ord> RbTree<V> {
	pub fn new() -> Self {
		Self { root: Ptr::null() }
	}

	pub fn root(&self) -> Ptr<RbTreeNode<V>> {
		self.root
	}

	pub fn insert(
		&mut self,
		n: Ptr<RbTreeNode<V>>,
		search: &mut RbTreeSearch<V>,
	) -> Option<Ptr<RbTreeNode<V>>> {
		let pair = search(self.root, n);
		let ret = self.insert_impl(n, pair);
		if ret.is_none() {
			self.insert_fixup(n);
		}
		ret
	}

	pub fn remove(
		&mut self,
		n: Ptr<RbTreeNode<V>>,
		search: &mut RbTreeSearch<V>,
	) -> Option<Ptr<RbTreeNode<V>>> {
		let pair = search(self.root, n);
		if pair.cur.is_null() {
			return None;
		}
		let ret = pair.cur.clone();
		self.remove_impl(pair);
		Some(ret)
	}

	fn remove_impl(&mut self, pair: RbNodePair<V>) {
		let node_to_delete = pair.cur;
		let mut do_fixup = node_to_delete.is_black();
		let (x, p, w);
		if node_to_delete.left.is_null() {
			x = node_to_delete.right;
			self.remove_transplant(node_to_delete, x);
			p = node_to_delete.parent;
			if !p.is_null() {
				if p.left.is_null() {
					w = p.right;
				} else {
					w = p.left;
				}
			} else {
				w = Ptr::null();
				do_fixup = false;
				if !self.root.is_null() {
					self.root.set_color(Color::Black);
				}
			}
		} else if node_to_delete.right.is_null() {
			x = node_to_delete.left;
			self.remove_transplant(node_to_delete, node_to_delete.left);
			p = node_to_delete.parent;
			if !p.is_null() {
				w = p.left;
			} else {
				w = Ptr::null();
			}
		} else {
			let mut successor = self.find_successor(node_to_delete);
			do_fixup = successor.is_black();
			x = successor.right;
			if !successor.parent.right.is_null() {
				if successor.parent.right.parent == node_to_delete {
					w = node_to_delete.left;
					p = successor;
				} else {
					w = successor.parent.right;
					p = w.parent;
				}
			} else {
				w = Ptr::null();
				p = Ptr::null();
			}

			if successor.parent != node_to_delete {
				self.remove_transplant(successor, successor.right);
				successor.right = node_to_delete.right;
				if !successor.right.is_null() {
					let successor_clone = successor.clone();
					successor.right.set_parent(successor_clone);
				}
			}

			self.remove_transplant(node_to_delete, successor);
			successor.left = node_to_delete.left;
			let successor_clone = successor.clone();
			successor.left.set_parent(successor_clone);
			if node_to_delete.is_black() {
				successor.set_color(Color::Black);
			} else {
				successor.set_color(Color::Red);
			}
		}
		if do_fixup {
			self.remove_fixup(p, w, x);
		}
	}

	fn find_successor(&mut self, mut x: Ptr<RbTreeNode<V>>) -> Ptr<RbTreeNode<V>> {
		x = x.right;
		loop {
			if x.left.is_null() {
				return x;
			}
			x = x.left;
		}
	}

	fn remove_transplant(&mut self, mut dst: Ptr<RbTreeNode<V>>, mut src: Ptr<RbTreeNode<V>>) {
		if dst.parent.is_null() {
			self.root = src;
		} else if dst == dst.parent.left {
			dst.parent.left = src;
		} else {
			dst.parent.right = src;
		}
		if !src.is_null() {
			src.set_parent(dst.parent);
		}
	}

	fn set_color_of_parent(&mut self, mut child: Ptr<RbTreeNode<V>>, parent: Ptr<RbTreeNode<V>>) {
		match parent.is_red() {
			true => child.set_color(Color::Red),
			false => child.set_color(Color::Black),
		}
	}

	fn is_root(&self, x: Ptr<RbTreeNode<V>>) -> bool {
		match x.is_null() {
			true => false,
			false => x.is_root(),
		}
	}

	fn is_black(&self, x: Ptr<RbTreeNode<V>>) -> bool {
		match x.is_null() {
			true => true,
			false => x.is_black(),
		}
	}

	fn is_red(&self, x: Ptr<RbTreeNode<V>>) -> bool {
		!self.is_black(x)
	}

	fn remove_fixup(
		&mut self,
		mut p: Ptr<RbTreeNode<V>>,
		mut w: Ptr<RbTreeNode<V>>,
		mut x: Ptr<RbTreeNode<V>>,
	) {
		while !self.is_root(x) && self.is_black(x) {
			if w == p.right {
				if self.is_red(w) {
					w.set_color(Color::Black);
					p.set_color(Color::Red);
					self.rotate_left(p);
					w = p.right;
				}
				if (w.left.is_null() || w.left.is_black())
					&& (w.right.is_null() || w.right.is_black())
				{
					w.set_color(Color::Red);
					x = p;
					p = p.parent;
					if !p.is_null() {
						let pl = p.left;
						if x == pl {
							w = p.right;
						} else {
							w = pl;
						}
					} else {
						w = Ptr::null();
					}
				} else {
					if w.right.is_null() || w.right.is_black() {
						w.left.set_color(Color::Black);
						w.set_color(Color::Red);
						self.rotate_right(w);
						w = p.right;
					}
					self.set_color_of_parent(w, p);
					p.set_color(Color::Black);
					w.right.set_color(Color::Black);
					self.rotate_left(p);
					x = self.root;
				}
			} else {
				if !w.is_null() && w.is_red() {
					w.set_color(Color::Black);
					p.set_color(Color::Red);
					self.rotate_right(p);
					w = p.left;
				}
				if (w.left.is_null() || w.left.is_black())
					&& (w.right.is_null() || w.right.is_black())
				{
					w.set_color(Color::Red);
					x = p;
					p = p.parent;
					if !p.is_null() {
						let pl = p.left;
						if x == pl {
							w = p.right;
						} else {
							w = pl;
						}
					} else {
						w = Ptr::null();
					}
				} else {
					if w.left.is_null() || w.left.is_black() {
						w.right.set_color(Color::Black);
						w.set_color(Color::Red);
						self.rotate_left(w);
						w = p.left;
					}
					self.set_color_of_parent(w, p);
					p.set_color(Color::Black);
					w.left.set_color(Color::Black);
					self.rotate_right(p);
					x = self.root;
				}
			}
		}
		if !x.is_null() {
			x.set_color(Color::Black);
		}
	}

	fn insert_impl(
		&mut self,
		mut n: Ptr<RbTreeNode<V>>,
		mut pair: RbNodePair<V>,
	) -> Option<Ptr<RbTreeNode<V>>> {
		let mut ret = None;
		if pair.cur.is_null() {
			n.set_parent(pair.parent);
			if pair.parent.is_null() {
				self.root = n;
			} else {
				match pair.is_right {
					true => pair.parent.right = n,
					false => pair.parent.left = n,
				}
			}
		} else {
			self.insert_transplant(pair.cur, n);
			if self.is_root(pair.cur) {
				self.root = n;
			}
			ret = Some(pair.cur);
		}
		ret
	}

	fn insert_transplant(&mut self, mut prev: Ptr<RbTreeNode<V>>, mut next: Ptr<RbTreeNode<V>>) {
		next.set_parent(prev.parent);
		next.right = prev.right;
		next.left = prev.left;
		if prev.is_red() {
			next.set_color(Color::Red);
		} else {
			next.set_color(Color::Black);
		}
		if !prev.parent.is_null() {
			if prev.parent.right == prev {
				prev.parent.right = next;
			} else {
				prev.parent.left = next;
			}
		}
		if !prev.right.is_null() {
			prev.right.parent = next;
		}

		if !prev.left.is_null() {
			prev.left.parent = next;
		}
	}

	fn rotate_left(&mut self, mut x: Ptr<RbTreeNode<V>>) {
		let mut y = x.right;
		x.right = y.left;
		if !y.left.is_null() {
			y.left.set_parent(x);
		}
		y.set_parent(x.parent);
		if x.parent.is_null() {
			self.root = y;
		} else if x == x.parent.left {
			x.parent.left = y;
		} else {
			x.parent.right = y;
		}
		y.left = x;
		x.set_parent(y);
	}

	fn rotate_right(&mut self, mut x: Ptr<RbTreeNode<V>>) {
		let mut y = x.left;
		x.left = y.right;
		if !y.right.is_null() {
			y.right.set_parent(x);
		}
		y.set_parent(x.parent);
		if x.parent.is_null() {
			self.root = y;
		} else if x == x.parent.right {
			x.parent.right = y;
		} else {
			x.parent.left = y;
		}
		y.right = x;
		x.set_parent(y);
	}

	fn insert_fixup(&mut self, mut k: Ptr<RbTreeNode<V>>) {
		let (mut parent, mut uncle, mut gparent);
		while !k.is_root() && k.parent.is_red() {
			parent = k.parent;
			gparent = parent.parent;
			if parent == gparent.left {
				uncle = gparent.right;
				if !uncle.is_null() && uncle.is_red() {
					parent.set_color(Color::Black);
					uncle.set_color(Color::Black);
					gparent.set_color(Color::Red);
					k = gparent
				} else {
					if k == parent.right {
						k = k.parent;
						self.rotate_left(k);
					}
					parent = k.parent;
					gparent = parent.parent;
					parent.set_color(Color::Black);
					gparent.set_color(Color::Red);
					self.rotate_right(gparent);
				}
			} else {
				uncle = gparent.left;
				if !uncle.is_null() && uncle.is_red() {
					parent.set_color(Color::Black);
					uncle.set_color(Color::Black);
					gparent.set_color(Color::Red);
					k = gparent;
				} else {
					if k == parent.left {
						k = k.parent;
						self.rotate_right(k);
					}
					parent = k.parent;
					gparent = parent.parent;
					parent.set_color(Color::Black);
					gparent.set_color(Color::Red);
					self.rotate_left(gparent);
				}
			}
		}
		self.root.set_color(Color::Black);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::murmur32::murmur3_32_of_u64;

	fn validate_node(
		node: Ptr<RbTreeNode<u64>>,
		mut black_count: Ptr<i32>,
		mut current_black_count: i32,
	) {
		if node.is_null() {
			if *black_count == 0 {
				*black_count = current_black_count;
			} else {
				assert_eq!(current_black_count, *black_count);
			}
			return;
		}

		if node.is_black() {
			current_black_count += 1;
		} else {
			if !node.parent.is_black() {
				println!("red/black violation node={}", node);
			}
			assert!(node.parent.is_black());
		}
		validate_node(node.right, black_count, current_black_count);
		validate_node(node.left, black_count, current_black_count);
	}

	fn validate_tree(root: Ptr<RbTreeNode<u64>>) {
		let black_count = Ptr::alloc(0).unwrap();
		if !root.is_null() {
			assert!(root.is_black());
			validate_node(root, black_count, 0);
		}
		black_count.release();
	}

	#[allow(dead_code)]
	fn print_node(node: Ptr<RbTreeNode<u64>>, depth: usize) {
		if node.is_null() {
			for _ in 0..depth {
				print!("    ");
			}
			println!("0 (B)");
			return;
		}

		print_node((*node).right, depth + 1);
		for _ in 0..depth {
			print!("    ");
		}
		println!(
			"{} {} ({})",
			node,
			node.value,
			if node.is_red() { "R" } else { "B" }
		);
		print_node((*node).left, depth + 1);
	}

	#[allow(dead_code)]
	fn print_tree(root: Ptr<RbTreeNode<u64>>) {
		if root.is_null() {
			println!("Red-Black Tree (root = 0) Empty Tree!");
		} else {
			println!("Red-Black Tree (root = {})", root);
			println!("===================================");
			print_node(root, 0);
			println!("===================================");
		}
	}

	#[test]
	fn test_rbtree1() {
		let mut tree = RbTree::new();

		let mut search = move |base: Ptr<RbTreeNode<u64>>, value: Ptr<RbTreeNode<u64>>| {
			let mut is_right = false;
			let mut cur = base;
			let mut parent = Ptr::null();

			while !cur.is_null() {
				let cmp = (*value).value.compare(&(*cur).value);
				if cmp == 0 {
					break;
				} else if cmp < 0 {
					parent = cur;
					is_right = false;
					cur = cur.left;
				} else {
					parent = cur;
					is_right = true;
					cur = cur.right;
				}
			}

			RbNodePair {
				cur,
				parent,
				is_right,
			}
		};

		let size = 100;
		let initial = unsafe { crate::ffi::getalloccount() };
		for x in 0..5 {
			let seed = 0x1234 + x;
			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let next = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				assert!(tree.insert(next, &mut search).is_none());
				validate_tree(tree.root());
			}

			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				let res = search(tree.root(), ptr);
				assert!(!res.cur.is_null());
				assert_eq!((*(res.cur)).value, v as u64);
				ptr.release();
			}

			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				let res = tree.remove(ptr, &mut search);
				validate_tree(tree.root());
				res.unwrap().release();
				let res = search(tree.root(), ptr);
				assert!(res.cur.is_null());
				ptr.release();
			}

			let seed = seed + 1;

			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let next = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				assert!(tree.insert(next, &mut search).is_none());
				validate_tree(tree.root());
			}

			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				let res = search(tree.root(), ptr);
				assert!(!res.cur.is_null());
				assert_eq!((*(res.cur)).value, v as u64);
				ptr.release();
			}

			let mut c = 0;

			for i in 0..size / 2 {
				c += 1;
				let v = murmur3_32_of_u64(i, seed);
				let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				let res = tree.remove(ptr, &mut search);
				validate_tree(tree.root());
				res.unwrap().release();
				let res = search(tree.root(), ptr);
				assert!(res.cur.is_null());
				ptr.release();
			}

			let seed = seed + 1;

			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let next = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				assert!(tree.insert(next, &mut search).is_none());
				validate_tree(tree.root());
			}

			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				let res = search(tree.root(), ptr);
				assert!(!res.cur.is_null());
				assert_eq!((*(res.cur)).value, v as u64);
				ptr.release();
			}

			for i in 0..size {
				let v = murmur3_32_of_u64(i, seed);
				let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				let res = tree.remove(ptr, &mut search);
				validate_tree(tree.root());
				res.unwrap().release();
				let res = search(tree.root(), ptr);
				assert!(res.cur.is_null());
				ptr.release();
			}

			let seed = seed - 1;
			for i in (size / 2)..size {
				c += 1;
				let v = murmur3_32_of_u64(i, seed);
				let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
				let res = tree.remove(ptr, &mut search);
				validate_tree(tree.root());
				res.unwrap().release();
				let res = tree.remove(ptr, &mut search);
				assert!(res.is_none());
				ptr.release();
			}
			assert_eq!(c, size);
		}
		assert_eq!(initial, unsafe { crate::ffi::getalloccount() });
	}

	#[derive(Debug, PartialEq, Clone)]
	struct TestTransplant {
		x: u64,
		y: u64,
	}

	impl Ord for TestTransplant {
		fn compare(&self, other: &Self) -> i8 {
			self.x.compare(&other.x)
		}
	}

	#[test]
	fn test_transplant() {
		let mut tree = RbTree::new();

		let mut search = move |base: Ptr<RbTreeNode<TestTransplant>>,
		                       value: Ptr<RbTreeNode<TestTransplant>>| {
			let mut is_right = false;
			let mut cur = base;
			let mut parent = Ptr::null();

			while !cur.is_null() {
				let cmp = (*value).value.compare(&(*cur).value);
				if cmp == 0 {
					break;
				} else if cmp < 0 {
					parent = cur;
					is_right = false;
					cur = cur.left;
				} else {
					parent = cur;
					is_right = true;
					cur = cur.right;
				}
			}

			RbNodePair {
				cur,
				parent,
				is_right,
			}
		};

		let initial = unsafe { crate::ffi::getalloccount() };
		{
			let size = 3;
			for i in 0..size {
				let v = TestTransplant { x: i, y: i };
				let next = Ptr::alloc(RbTreeNode::new(v)).unwrap();
				let res = tree.insert(next, &mut search);
				assert!(res.is_none());
			}

			for i in 0..size {
				let v = TestTransplant { x: i, y: i };
				let ptr = Ptr::alloc(RbTreeNode::new(v.clone())).unwrap();
				let res = search(tree.root(), ptr);
				assert!(!res.cur.is_null());
				assert_eq!((*(res.cur)).value, v);
				ptr.release();
			}

			for i in 0..size {
				let v = TestTransplant { x: i, y: i + 1 };
				let next = Ptr::alloc(RbTreeNode::new(v)).unwrap();
				let res = tree.insert(next, &mut search);
				assert!(res.is_some());
				res.unwrap().release();
			}

			for i in 0..size {
				let v = TestTransplant { x: i, y: i + 1 };
				let ptr = Ptr::alloc(RbTreeNode::new(v.clone())).unwrap();
				let res = search(tree.root(), ptr);
				assert!(!res.cur.is_null());
				assert_eq!((*(res.cur)).value, v);
				ptr.release();
			}

			for i in 0..size {
				let v = TestTransplant { x: i, y: i + 91 };
				let ptr = Ptr::alloc(RbTreeNode::new(v)).unwrap();
				let res = tree.remove(ptr, &mut search);
				res.unwrap().release();
				let res = search(tree.root(), ptr);
				assert!(res.cur.is_null());
				ptr.release();
			}

			for i in 0..size {
				let v = TestTransplant { x: i, y: i + 10 };
				let next = Ptr::alloc(RbTreeNode::new(v)).unwrap();
				let res = tree.insert(next, &mut search);
				assert!(res.is_none());
			}

			for i in 0..size {
				let v = TestTransplant { x: i, y: i + 10 };
				let ptr = Ptr::alloc(RbTreeNode::new(v.clone())).unwrap();
				let res = search(tree.root(), ptr);
				assert!(!res.cur.is_null());
				assert_eq!((*(res.cur)).value, v);
				ptr.release();
			}

			for i in 0..size {
				let v = TestTransplant { x: i, y: i + 91 };
				let ptr = Ptr::alloc(RbTreeNode::new(v)).unwrap();
				let res = tree.remove(ptr, &mut search);
				res.unwrap().release();
				let res = search(tree.root(), ptr);
				assert!(res.cur.is_null());
				ptr.release();
			}
		}
		assert_eq!(initial, unsafe { crate::ffi::getalloccount() });
	}
}
