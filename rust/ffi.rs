// # FFI bindings
// Direct bindings to the underlying C library functions.

#![allow(dead_code)]
#![allow(invalid_value)]

use secp256k1::types::*;

extern "C" {
	pub static secp256k1_nonce_function_rfc6979: NonceFn;

	pub static secp256k1_nonce_function_default: NonceFn;

	// Contexts
	pub fn secp256k1_context_create(flags: u32) -> *mut Context;

	pub fn secp256k1_context_clone(cx: *mut Context) -> *mut Context;

	pub fn secp256k1_context_destroy(cx: *mut Context);

	pub fn secp256k1_context_randomize(cx: *mut Context, seed32: *const u8) -> i32;
	// Scratch space
	pub fn secp256k1_scratch_space_create(cx: *mut Context, max_size: usize) -> *mut ScratchSpace;

	pub fn secp256k1_scratch_space_destroy(sp: *mut ScratchSpace);

	// Generator
	pub fn secp256k1_generator_generate(
		cx: *const Context,
		gen: *mut Generator,
		seed32: *const u8,
	) -> i32;

	// TODO secp256k1_context_set_illegal_callback
	// TODO secp256k1_context_set_error_callback
	// (Actually, I don't really want these exposed; if either of these
	// are ever triggered it indicates a bug in rust-secp256k1, since
	// one goal is to use Rust's type system to eliminate all possible
	// bad inputs.)

	// Pubkeys
	pub fn secp256k1_ec_pubkey_parse(
		cx: *const Context,
		pk: *mut PublicKey,
		input: *const u8,
		in_len: u64,
	) -> i32;

	pub fn secp256k1_ec_pubkey_serialize(
		cx: *const Context,
		output: *const u8,
		out_len: *mut u64,
		pk: *const PublicKey,
		compressed: u32,
	) -> i32;

	// Signatures
	pub fn secp256k1_ecdsa_signature_parse_der(
		cx: *const Context,
		sig: *mut Signature,
		input: *const u8,
		in_len: u64,
	) -> i32;

	pub fn secp256k1_ecdsa_signature_parse_compact(
		cx: *const Context,
		sig: *mut Signature,
		input64: *const u8,
	) -> i32;

	pub fn ecdsa_signature_parse_der_lax(
		cx: *const Context,
		sig: *mut Signature,
		input: *const u8,
		in_len: u64,
	) -> i32;

	pub fn secp256k1_ecdsa_signature_serialize_der(
		cx: *const Context,
		output: *const u8,
		out_len: *mut u64,
		sig: *const Signature,
	) -> i32;

	pub fn secp256k1_ecdsa_signature_serialize_compact(
		cx: *const Context,
		output64: *const u8,
		sig: *const Signature,
	) -> i32;

	pub fn secp256k1_ecdsa_recoverable_signature_parse_compact(
		cx: *const Context,
		sig: *mut RecoverableSignature,
		input64: *const u8,
		recid: i32,
	) -> i32;

	pub fn secp256k1_ecdsa_recoverable_signature_serialize_compact(
		cx: *const Context,
		output64: *const u8,
		recid: *mut i32,
		sig: *const RecoverableSignature,
	) -> i32;

	pub fn secp256k1_ecdsa_recoverable_signature_convert(
		cx: *const Context,
		sig: *mut Signature,
		input: *const RecoverableSignature,
	) -> i32;

	pub fn secp256k1_ecdsa_signature_normalize(
		cx: *const Context,
		out_sig: *mut Signature,
		in_sig: *const Signature,
	) -> i32;

	// ECDSA
	pub fn secp256k1_ecdsa_verify(
		cx: *const Context,
		sig: *const Signature,
		msg32: *const u8,
		pk: *const PublicKey,
	) -> i32;

	pub fn secp256k1_ecdsa_sign(
		cx: *const Context,
		sig: *mut Signature,
		msg32: *const u8,
		sk: *const u8,
		noncefn: NonceFn,
		noncedata: *const u8,
	) -> i32;

	pub fn secp256k1_ecdsa_sign_recoverable(
		cx: *const Context,
		sig: *mut RecoverableSignature,
		msg32: *const u8,
		sk: *const u8,
		noncefn: NonceFn,
		noncedata: *const u8,
	) -> i32;

	pub fn secp256k1_ecdsa_recover(
		cx: *const Context,
		pk: *mut PublicKey,
		sig: *const RecoverableSignature,
		msg32: *const u8,
	) -> i32;
	// AGGSIG (Schnorr) Multisig
	pub fn secp256k1_aggsig_context_create(
		cx: *const Context,
		pks: *const PublicKey,
		n_pks: usize,
		seed32: *const u8,
	) -> *mut AggSigContext;

	pub fn secp256k1_aggsig_context_destroy(aggctx: *mut AggSigContext);

	pub fn secp256k1_aggsig_generate_nonce(
		cx: *const Context,
		aggctx: *mut AggSigContext,
		index: usize,
	) -> i32;

	pub fn secp256k1_aggsig_partial_sign(
		cx: *const Context,
		aggctx: *mut AggSigContext,
		sig: *mut AggSigPartialSignature,
		msghash32: *const Message,
		seckey32: *const SecretKey,
		index: usize,
	) -> i32;

	pub fn secp256k1_aggsig_combine_signatures(
		cx: *const Context,
		aggctx: *mut AggSigContext,
		sig64: *mut Signature,
		partial: *const AggSigPartialSignature,
		index: usize,
	) -> i32;

	pub fn secp256k1_aggsig_build_scratch_and_verify(
		cx: *const Context,
		sig64: *const Signature,
		msg32: *const Message,
		pks: *const PublicKey,
		n_pubkeys: usize,
	) -> i32;

	// AGGSIG (single sig or single-signer Schnorr)
	pub fn secp256k1_aggsig_export_secnonce_single(
		cx: *const Context,
		secnonce32: *mut SecretKey,
		seed32: *const u8,
	) -> i32;

	pub fn secp256k1_aggsig_sign_single(
		cx: *const Context,
		sig: *mut Signature,
		msg32: *const Message,
		seckey32: *const SecretKey,
		secnonce32: *const u8,
		extra32: *const u8,
		pubnonce_for_e: *const PublicKey,
		pubnonce_total: *const PublicKey,
		pubkey_for_e: *const PublicKey,
		seed32: *const u8,
	) -> i32;

	pub fn secp256k1_aggsig_verify_single(
		cx: *const Context,
		sig: *const Signature,
		msg32: *const Message,
		pubnonce: *const PublicKey,
		pk: *const PublicKey,
		pk_total: *const PublicKey,
		extra_pubkey: *const PublicKey,
		is_partial: u32,
	) -> i32;

	pub fn secp256k1_schnorrsig_verify_batch(
		cx: *const Context,
		scratch: *mut ScratchSpace,
		sig: *const *const u8,
		msg32: *const *const u8,
		pk: *const *const PublicKey,
		n_sigs: usize,
	) -> i32;

	pub fn secp256k1_aggsig_add_signatures_single(
		cx: *const Context,
		ret_sig: *mut Signature,
		sigs: *const *const u8,
		num_sigs: usize,
		pubnonce_total: *const PublicKey,
	) -> i32;

	pub fn secp256k1_aggsig_subtract_partial_signature(
		cx: *const Context,
		ret_partsig: *mut Signature,
		ret_partsig_alt: *mut Signature,
		sig: *const Signature,
		part_sig: *const Signature,
	) -> i32;

	// EC
	pub fn secp256k1_ec_seckey_verify(cx: *const Context, sk: *const u8) -> i32;

	pub fn secp256k1_ec_pubkey_create(cx: *const Context, pk: *mut PublicKey, sk: *const u8)
		-> i32;

	pub fn secp256k1_ec_privkey_tweak_add(cx: *const Context, sk: *mut u8, tweak: *const u8)
		-> i32;

	pub fn secp256k1_ec_pubkey_tweak_add(
		cx: *const Context,
		pk: *mut PublicKey,
		tweak: *const u8,
	) -> i32;

	pub fn secp256k1_ec_privkey_tweak_mul(cx: *const Context, sk: *mut u8, tweak: *const u8)
		-> i32;

	pub fn secp256k1_ec_pubkey_tweak_mul(
		cx: *const Context,
		pk: *mut PublicKey,
		tweak: *const u8,
	) -> i32;

	pub fn secp256k1_ec_pubkey_combine(
		cx: *const Context,
		out: *mut PublicKey,
		ins: *const *const PublicKey,
		n: i32,
	) -> i32;

	pub fn secp256k1_ec_privkey_tweak_inv(cx: *const Context, sk: *mut u8) -> i32;

	pub fn secp256k1_ec_privkey_tweak_neg(cx: *const Context, sk: *mut u8) -> i32;

	pub fn secp256k1_ecdh(
		cx: *const Context,
		out: *mut SharedSecret,
		point: *const PublicKey,
		scalar: *const u8,
	) -> i32;

	// Parse a 33-byte commitment into 64 byte internal commitment object
	pub fn secp256k1_pedersen_commitment_parse(
		cx: *const Context,
		commit: *mut u8,
		input: *const u8,
	) -> i32;

	// Serialize a 64-byte commit object into a 33 byte serialized byte sequence
	pub fn secp256k1_pedersen_commitment_serialize(
		cx: *const Context,
		output: *mut u8,
		commit: *const u8,
	) -> i32;

	// Generates a pedersen commitment: *commit = blind * G + value * G2.
	// The commitment is 33 bytes, the blinding factor is 32 bytes.
	pub fn secp256k1_pedersen_commit(
		ctx: *const Context,
		commit: *mut u8,
		blind: *const u8,
		value: u64,
		value_gen: *const u8,
		blind_gen: *const u8,
	) -> i32;

	// Generates a pedersen commitment: *commit = blind * G + value * G2.
	// The commitment is 33 bytes, the blinding factor and the value are 32 bytes.
	pub fn secp256k1_pedersen_blind_commit(
		ctx: *const Context,
		commit: *mut u8,
		blind: *const u8,
		value: *const u8,
		value_gen: *const u8,
		blind_gen: *const u8,
	) -> i32;

	// Get the public key of a pedersen commitment
	pub fn secp256k1_pedersen_commitment_to_pubkey(
		cx: *const Context,
		pk: *mut PublicKey,
		commit: *const u8,
	) -> i32;

	// Get a pedersen commitment from a pubkey
	pub fn secp256k1_pubkey_to_pedersen_commitment(
		cx: *const Context,
		commit: *mut u8,
		pk: *const PublicKey,
	) -> i32;

	// Takes a list of n pointers to 32 byte blinding values, the first negs
	// of which are treated with positive sign and the rest negative, then
	// calculates an additional blinding value that adds to zero.
	pub fn secp256k1_pedersen_blind_sum(
		ctx: *const Context,
		blind_out: *const u8,
		blinds: *const *const u8,
		n: u64,
		npositive: u64,
	) -> i32;

	// Takes two list of 64-byte commitments and sums the first set, subtracts
	// the second and returns the resulting commitment.
	pub fn secp256k1_pedersen_commit_sum(
		ctx: *const Context,
		commit_out: *const u8,
		commits: *const *const u8,
		pcnt: u64,
		ncommits: *const *const u8,
		ncnt: u64,
	) -> i32;

	// Calculate blinding factor for switch commitment x + H(xG+vH | xJ)
	pub fn secp256k1_blind_switch(
		ctx: *const Context,
		blind_switch: *mut u8,
		blind: *const u8,
		value: u64,
		value_gen: *const u8,
		blind_gen: *const u8,
		switch_pubkey: *const u8,
	) -> i32;

	// Takes two list of 64-byte commitments and sums the first set and
	// subtracts the second and verifies that they sum to 0.
	pub fn secp256k1_pedersen_verify_tally(
		ctx: *const Context,
		commits: *const *const u8,
		pcnt: u64,
		ncommits: *const *const u8,
		ncnt: u64,
	) -> i32;

	pub fn secp256k1_rangeproof_info(
		ctx: *const Context,
		exp: *mut i32,
		mantissa: *mut i32,
		min_value: *mut u64,
		max_value: *mut u64,
		proof: *const u8,
		plen: u64,
	) -> i32;

	pub fn secp256k1_rangeproof_rewind(
		ctx: *const Context,
		blind_out: *mut u8,
		value_out: *mut u64,
		message_out: *mut u8,
		outlen: *mut u64,
		nonce: *const u8,
		min_value: *mut u64,
		max_value: *mut u64,
		commit: *const u8,
		proof: *const u8,
		plen: u64,
		extra_commit: *const u8,
		extra_commit_len: u64,
		gen: *const u8,
	) -> i32;

	pub fn secp256k1_rangeproof_verify(
		ctx: *const Context,
		min_value: &mut u64,
		max_value: &mut u64,
		commit: *const u8,
		proof: *const u8,
		plen: u64,
		extra_commit: *const u8,
		extra_commit_len: u64,
		gen: *const u8,
	) -> i32;

	pub fn secp256k1_rangeproof_sign(
		ctx: *const Context,
		proof: *mut u8,
		plen: *mut u64,
		min_value: u64,
		commit: *const u8,
		blind: *const u8,
		nonce: *const u8,
		exp: i32,
		min_bits: i32,
		value: u64,
		message: *const u8,
		msg_len: u64,
		extra_commit: *const u8,
		extra_commit_len: u64,
		gen: *const u8,
	) -> i32;

	pub fn secp256k1_bulletproof_generators_create(
		ctx: *const Context,
		blinding_gen: *const u8,
		n: u64,
	) -> *mut BulletproofGenerators;

	pub fn secp256k1_bulletproof_generators_destroy(
		ctx: *const Context,
		gen: *mut BulletproofGenerators,
	);

	pub fn secp256k1_bulletproof_rangeproof_prove(
		ctx: *const Context,
		scratch: *mut ScratchSpace,
		gens: *const BulletproofGenerators,
		proof: *mut u8,
		plen: *mut u64,
		tau_x: *mut u8,
		t_one: *mut PublicKey,
		t_two: *mut PublicKey,
		value: *const u64,
		min_value: *const u64,
		blind: *const *const u8,
		commits: *const *const u8,
		n_commits: u64,
		value_gen: *const u8,
		nbits: u64,
		nonce: *const u8,
		private_nonce: *const u8,
		extra_commit: *const u8,
		extra_commit_len: u64,
		message: *const u8,
	) -> i32;

	pub fn secp256k1_bulletproof_rangeproof_verify(
		ctx: *const Context,
		scratch: *mut ScratchSpace,
		gens: *const BulletproofGenerators,
		proof: *const u8,
		plen: u64,
		min_value: *const u64,
		commit: *const u8,
		n_commits: u64,
		nbits: u64,
		value_gen: *const u8,
		extra_commit: *const u8,
		extra_commit_len: u64,
	) -> i32;

	pub fn secp256k1_bulletproof_rangeproof_verify_multi(
		ctx: *const Context,
		scratch: *mut ScratchSpace,
		gens: *const BulletproofGenerators,
		proofs: *const *const u8,
		n_proofs: u64,
		plen: u64,
		min_value: *const *const u64,
		commits: *const *const u8,
		n_commits: u64,
		nbits: u64,
		value_gen: *const u8,
		extra_commit: *const *const u8,
		extra_commit_len: *const u64,
	) -> i32;

	pub fn secp256k1_bulletproof_rangeproof_rewind(
		ctx: *const Context,
		value: *mut u64,
		blind: *mut u8,
		proof: *const u8,
		plen: u64,
		min_value: u64,
		commit: *const u8,
		value_gen: *const u8,
		nonce: *const u8,
		extra_commit: *const u8,
		extra_commit_len: u64,
		message: *mut u8,
	) -> i32;

	// MISC
	pub fn rand_bytes(data: *mut u8, len: usize) -> i32;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn alloc(len: usize) -> *const u8;
	pub fn resize(ptr: *const u8, len: usize) -> *const u8;
	pub fn release(ptr: *const u8);
	pub fn sleep_millis(millis: u64) -> i32;
	pub fn ptr_add(p: *mut u8, v: i64);
	pub fn getalloccount() -> i64;
	pub fn getfdcount() -> i64;
	pub fn atomic_store_u64(ptr: *mut u64, value: u64);
	pub fn atomic_load_u64(ptr: *const u64) -> u64;
	pub fn atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool;
	pub fn f64_to_str(d: f64, buf: *mut u8, capacity: u64) -> i32;
	pub fn sched_yield() -> i32;
	pub fn cstring_len(s: *const u8) -> usize;
	pub fn backtrace_ptr(bin: *const u8, len: usize) -> usize;
	pub fn backtrace_to_string(bt: *const u8, bin: *const u8) -> *const u8;
	pub fn backtrace_size() -> usize;
	pub fn backtrace_free(bt: *const u8);
	pub fn getmicros() -> i64;

	// THREAD
	pub fn thread_create(start_routine: extern "C" fn(*mut u8), arg: *mut u8) -> i32;
	pub fn thread_create_joinable(
		handle: *const u8,
		start_routine: extern "C" fn(*mut u8),
		arg: *mut u8,
	) -> i32;
	pub fn thread_join(handle: *const u8) -> i32;
	pub fn thread_detach(handle: *const u8) -> i32;
	pub fn thread_handle_size() -> usize;

	// CHANNEL
	pub fn channel_init(channel: *const u8) -> i32;
	pub fn channel_send(channel: *const u8, ptr: *const u8) -> i32;
	pub fn channel_recv(channel: *const u8) -> *mut u8;
	pub fn channel_handle_size() -> usize;
	pub fn channel_destroy(channel: *const u8) -> i32;
	pub fn channel_pending(channel: *const u8) -> bool;

	// SOCKET
	pub fn socket_handle_size() -> usize;
	pub fn socket_event_size() -> usize;
	pub fn socket_multiplex_handle_size() -> usize;
	pub fn socket_fd(handle: *const u8) -> i32;
	pub fn socket_connect(handle: *mut u8, addr: *const u8, port: i32) -> i32;
	pub fn socket_shutdown(handle: *const u8) -> i32;
	pub fn socket_close(handle: *const u8) -> i32;
	pub fn socket_listen(handle: *mut u8, addr: *const u8, port: u16, backlog: i32) -> i32;
	pub fn socket_accept(handle: *const u8, nhandle: *mut u8) -> i32;
	pub fn socket_send(handle: *const u8, buf: *const u8, len: usize) -> i64;
	pub fn socket_recv(handle: *const u8, buf: *mut u8, capacity: usize) -> i64;
	pub fn socket_clear_pipe(handle: *const u8) -> i32;

	pub fn socket_multiplex_init(handle: *mut u8) -> i32;
	pub fn socket_multiplex_register(
		handle: *const u8,
		socket: *const u8,
		flags: i32,
		ptr: *const u8,
	) -> i32;
	pub fn socket_multiplex_unregister_write(
		handle: *const u8,
		socket: *const u8,
		connptr: *const u8,
	) -> i32;
	pub fn socket_multiplex_wait(
		handle: *const u8,
		events: *mut u8,
		max_events: i32,
		timeout_millis: i64,
	) -> i32;
	pub fn socket_event_handle(handle: *mut u8, event: *const u8);
	pub fn socket_event_is_read(event: *const u8) -> bool;
	pub fn socket_event_is_write(event: *const u8) -> bool;
	pub fn socket_event_ptr(event: *const u8) -> *const u8;
	pub fn socket_handle_eq(handle1: *const u8, handle2: *const u8) -> bool;

	pub fn open_pipe(pair: *mut u8) -> i32;
	pub fn Base64decode(output: *mut u8, input: *mut u8);
	pub fn Base64encode(input: *const u8, output: *mut u8, len: usize);
	pub fn SHA1(data: *const u8, size: usize, hash: *mut u8);

	// CPSRNG
	pub fn cpsrng_rand_bytes(v: *mut u8, len: usize);
	pub fn cpsrng_context_create() -> *mut u8;
	pub fn cpsrng_context_destroy(ctx: *mut u8);
	pub fn cpsrng_rand_bytes_ctx(ctx: *mut u8, v: *mut u8, len: usize);
}
