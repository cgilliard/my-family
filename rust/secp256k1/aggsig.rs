// Rust secp256k1 bindings for aggsig functions
// 2018 The Grin developers
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Aggregated Signature (a.k.a. Schnorr) Functionality

use core::ptr;
use ffi;
use ffi::cpsrng_rand_bytes_ctx;
use prelude::*;
use secp256k1::types::*;

const SCRATCH_SPACE_SIZE: usize = 1024 * 1024;

/// Single-Signer (plain old Schnorr, sans-multisig) export nonce
/// Returns: Ok(SecretKey) on success
/// In:
/// msg: the message to sign
/// seckey: the secret key
pub fn export_secnonce_single(secp: &Secp256k1, rand: *mut u8) -> Result<SecretKey, Error> {
	let mut return_key = SecretKey::generate(rand);
	let mut seed = [0u8; 32];
	unsafe { cpsrng_rand_bytes_ctx(rand, &mut seed as *mut u8, 32) };
	let retval = unsafe {
		ffi::secp256k1_aggsig_export_secnonce_single(
			secp.ctx,
			return_key.as_mut_ptr(),
			seed.as_ptr(),
		)
	};
	if retval == 0 {
		return Err(err!(InvalidSignature));
	}
	Ok(return_key)
}

// This is a macro that check zero public key
macro_rules! is_zero_pubkey {
	(reterr => $e:expr) => {
		match $e {
			Some(n) => {
				let mut is_ok = false;
				for i in 0..n.0.len() {
					if n.0[i] != 0 {
						is_ok = true;
					}
				}
				if !is_ok {
					return Err(err!(InvalidPublicKey));
				}
				n.as_ptr()
			}
			None => ptr::null(),
		}
	};
	(retfalse => $e:expr) => {
		match $e {
			Some(n) => {
				let mut is_ok = false;
				for i in 0..n.0.len() {
					if n.0[i] != 0 {
						is_ok = true;
					}
				}
				if !is_ok {
					return false;
				}
				n.as_ptr()
			}
			None => ptr::null(),
		}
	};
}

/// Single-Signer (plain old Schnorr, sans-multisig) signature creation
/// Returns: Ok(Signature) on success
/// In:
/// msg: the message to sign
/// seckey: the secret key
/// extra: if Some(), add this key to s
/// secnonce: if Some(SecretKey), the secret nonce to use. If None, generate a nonce
/// pubnonce: if Some(PublicKey), overrides the public nonce to encode as part of e
/// final_nonce_sum: if Some(PublicKey), overrides the public nonce to encode as part of e
pub fn sign_single(
	secp: &Secp256k1,
	msg: &Message,
	seckey: &SecretKey,
	secnonce: Option<&SecretKey>,
	extra: Option<&SecretKey>,
	pubnonce: Option<&PublicKey>,
	pubkey_for_e: Option<&PublicKey>,
	final_nonce_sum: Option<&PublicKey>,
	rand: *mut u8,
) -> Result<Signature, Error> {
	let mut retsig = Signature::from(Signature::new());
	let mut seed = [0u8; 32];
	unsafe { cpsrng_rand_bytes_ctx(rand, &mut seed as *mut u8, 32) };

	let secnonce = match secnonce {
		Some(n) => n.0.as_ptr(),
		None => ptr::null(),
	};

	let pubnonce = is_zero_pubkey!(reterr => pubnonce);

	let extra = match extra {
		Some(e) => e.0.as_ptr(),
		None => ptr::null(),
	};

	let final_nonce_sum = is_zero_pubkey!(reterr => final_nonce_sum);

	let pe = is_zero_pubkey!(reterr => pubkey_for_e);

	let retval = unsafe {
		ffi::secp256k1_aggsig_sign_single(
			secp.ctx,
			retsig.as_mut_ptr(),
			msg.as_ptr(),
			seckey.as_ptr(),
			secnonce,
			extra,
			pubnonce,
			final_nonce_sum,
			pe,
			seed.as_ptr(),
		)
	};
	if retval == 0 {
		return Err(err!(InvalidSignature));
	}
	Ok(retsig)
}

/// Single-Signer (plain old Schnorr, sans-multisig) signature verification
/// Returns: Ok(Signature) on success
/// In:
/// sig: The signature
/// msg: the message to verify
/// pubnonce: if Some(PublicKey) overrides the public nonce used to calculate e
/// pubkey: the public key
/// pubkey_total: The total of all public keys (for the message in e)
/// is_partial: whether this is a partial sig, or a fully-combined sig
pub fn verify_single(
	secp: &Secp256k1,
	sig: &Signature,
	msg: &Message,
	pubnonce: Option<&PublicKey>,
	pubkey: &PublicKey,
	pubkey_total_for_e: Option<&PublicKey>,
	extra_pubkey: Option<&PublicKey>,
	is_partial: bool,
) -> bool {
	let pubnonce = is_zero_pubkey!(retfalse => pubnonce);

	let pe = is_zero_pubkey!(retfalse => pubkey_total_for_e);

	let extra = is_zero_pubkey!(retfalse => extra_pubkey);

	let is_partial = match is_partial {
		true => 1,
		false => 0,
	};

	let mut is_ok = false;
	for i in 0..sig.0.len() {
		if sig.0[i] != 0 {
			is_ok = true;
		}
	}
	if !is_ok {
		return false;
	}

	let retval = unsafe {
		ffi::secp256k1_aggsig_verify_single(
			secp.ctx,
			sig.as_ptr(),
			msg.as_ptr(),
			pubnonce,
			pubkey.as_ptr(),
			pe,
			extra,
			is_partial,
		)
	};
	match retval {
		0 => false,
		1 => true,
		_ => false,
	}
}

/// Batch Schnorr signature verification
/// Returns: true on success
/// In:
/// sigs: The signatures
/// msg: The messages to verify
/// pubkey: The public keys
pub fn verify_batch(
	secp: &Secp256k1,
	sigs: &Vec<Signature>,
	msgs: &Vec<Message>,
	pub_keys: &Vec<PublicKey>,
) -> bool {
	if sigs.len() != msgs.len() || sigs.len() != pub_keys.len() {
		return false;
	}

	for i in 0..pub_keys.len() {
		let mut is_ok = false;
		for j in 0..pub_keys[i].0.len() {
			if pub_keys[i].0[j] != 0 {
				is_ok = true;
			}
		}
		if !is_ok {
			return false;
		}
	}

	let mut sigs_vec = Vec::new();
	for sig in sigs {
		match sigs_vec.push(sig.0.as_ptr()) {
			Ok(_) => {}
			Err(_) => return false,
		}
	}
	let mut msgs_vec = Vec::new();
	for msg in msgs {
		match msgs_vec.push(msg.0.as_ptr()) {
			Ok(_) => {}
			Err(_) => return false,
		}
	}

	let mut pub_keys_vec = Vec::new();
	for pk in pub_keys {
		match pub_keys_vec.push(pk.as_ptr()) {
			Ok(_) => {}
			Err(_) => return false,
		}
	}

	unsafe {
		let scratch = ffi::secp256k1_scratch_space_create(secp.ctx, SCRATCH_SPACE_SIZE);
		let result = ffi::secp256k1_schnorrsig_verify_batch(
			secp.ctx,
			scratch,
			sigs_vec.as_ptr() as *const *const u8,
			msgs_vec.as_ptr() as *const *const u8,
			pub_keys_vec.as_ptr() as *const *const PublicKey,
			sigs.len(),
		);
		ffi::secp256k1_scratch_space_destroy(scratch);
		result == 1
	}
}

/// Single-Signer addition of Signatures
/// Returns: Ok(Signature) on success
/// In:
/// sig1: sig1 to add
/// sig2: sig2 to add
/// pubnonce_total: sum of public nonces
pub fn add_signatures_single(
	secp: &Secp256k1,
	sigs: Vec<&Signature>,
	pubnonce_total: &PublicKey,
) -> Result<Signature, Error> {
	let mut retsig = Signature::new();

	let mut sig_vec = Vec::new();
	for sig in &sigs {
		match sig_vec.push(sig.0.as_ptr()) {
			Ok(_) => {}
			Err(_) => return Err(err!(Alloc)),
		}
	}
	let retval = unsafe {
		ffi::secp256k1_aggsig_add_signatures_single(
			secp.ctx,
			retsig.as_mut_ptr(),
			sig_vec.as_ptr() as *const *const u8,
			sig_vec.len(),
			pubnonce_total.as_ptr(),
		)
	};
	if retval == 0 {
		return Err(err!(InvalidSignature));
	}
	Ok(retsig)
}

/// Subtraction of partial signature from a signature
/// Returns: Ok((Signature, None)) on success if the resulting signature has only one possibility
///          Ok((Signature, Signature)) on success if the resulting signature could be one of either possiblity
/// In:
/// sig: completed signature from which to subtact a partial
/// partial_sig: the partial signature to subtract
pub fn subtract_partial_signature(
	secp: &Secp256k1,
	sig: &Signature,
	partial_sig: &Signature,
) -> Result<(Signature, Option<Signature>), Error> {
	let mut ret_partsig = Signature::new();
	let mut ret_partsig_alt = Signature::new();
	let retval = unsafe {
		ffi::secp256k1_aggsig_subtract_partial_signature(
			secp.ctx,
			ret_partsig.as_mut_ptr(),
			ret_partsig_alt.as_mut_ptr(),
			sig.as_ptr(),
			partial_sig.as_ptr(),
		)
	};

	match retval {
		-1 => Err(err!(SignatureSubtractionError)),
		1 => Ok((ret_partsig, None)),
		2 => Ok((ret_partsig, Some(ret_partsig_alt))),
		_ => Err(err!(InvalidSignature)),
	}
}

/// Manages an instance of an aggsig multisig context, and provides all methods
/// to act on that context
#[derive(Clone)]
pub struct AggSigContext {
	ctx: *mut Context,
	aggsig_ctx: *mut crate::secp256k1::types::AggSigContext,
}

impl AggSigContext {
	/// Creates new aggsig context with a new random seed
	pub fn new(
		secp: &Secp256k1,
		pubkeys_vec: &Vec<PublicKey>,
		rand: *mut u8,
	) -> Result<AggSigContext, Error> {
		let mut seed = [0u8; 32];
		unsafe { cpsrng_rand_bytes_ctx(rand, &mut seed as *mut u8, 32) };
		let mut pubkeys: Vec<*const PublicKey> = Vec::new();
		for pubkey in pubkeys_vec {
			match pubkeys.push(pubkey.as_ptr()) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}

		Ok(unsafe {
			AggSigContext {
				ctx: secp.ctx,
				aggsig_ctx: ffi::secp256k1_aggsig_context_create(
					secp.ctx,
					pubkeys[0],
					pubkeys.len(),
					seed.as_ptr(),
				),
			}
		})
	}

	/// Generate a nonce pair for a single signature part in an aggregated signature
	/// Returns: true on success
	///          false if a nonce has already been generated for this index
	/// In: index: which signature to generate a nonce for
	pub fn generate_nonce(&self, index: usize) -> bool {
		let retval =
			unsafe { ffi::secp256k1_aggsig_generate_nonce(self.ctx, self.aggsig_ctx, index) };
		match retval {
			0 => false,
			1 => true,
			_ => false,
		}
	}

	/// Generate a single signature part in an aggregated signature
	/// Returns: Ok(AggSigPartialSignature) on success
	/// In:
	/// msg: the message to sign
	/// seckey: the secret key
	/// index: which index to generate a partial sig for
	pub fn partial_sign(
		&self,
		msg: Message,
		seckey: SecretKey,
		index: usize,
	) -> Result<AggSigPartialSignature, Error> {
		let mut retsig = AggSigPartialSignature::new();
		let retval = unsafe {
			ffi::secp256k1_aggsig_partial_sign(
				self.ctx,
				self.aggsig_ctx,
				retsig.as_mut_ptr(),
				msg.as_ptr(),
				seckey.as_ptr(),
				index,
			)
		};
		if retval == 0 {
			return Err(err!(PartialSigFailure));
		}
		Ok(retsig)
	}

	/// Aggregate multiple signature parts into a single aggregated signature
	/// Returns: Ok(Signature) on success
	/// In:
	/// partial_sigs: vector of partial signatures
	pub fn combine_signatures(
		&self,
		partial_sigs: &Vec<AggSigPartialSignature>,
	) -> Result<Signature, Error> {
		let mut retsig = Signature::new();
		let mut partial_sigs_vec: Vec<*const AggSigPartialSignature> = Vec::new();
		for psig in partial_sigs {
			match partial_sigs_vec.push(psig.as_ptr()) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		if partial_sigs_vec.len() > 0 {
			let partial_sigs = &partial_sigs_vec[0..partial_sigs_vec.len()];
			let retval = unsafe {
				ffi::secp256k1_aggsig_combine_signatures(
					self.ctx,
					self.aggsig_ctx,
					retsig.as_mut_ptr(),
					partial_sigs[0],
					partial_sigs.len(),
				)
			};
			if retval == 0 {
				return Err(err!(PartialSigFailure));
			}
		} else {
			return Err(err!(IllegalArgument));
		}
		Ok(retsig)
	}

	/// Verifies aggregate sig
	/// Returns: true if valid, okay if not
	/// In:
	/// msg: message to verify
	/// sig: combined signature
	/// pks: public keys
	pub fn verify(&self, sig: Signature, msg: Message, pks_vec: &Vec<PublicKey>) -> bool {
		let mut pks: Vec<*const PublicKey> = Vec::new();
		for pk in pks_vec {
			match pks.push(pk.as_ptr()) {
				Ok(_) => {}
				Err(_) => return false,
			}
		}
		if pks.len() > 0 {
			let pks = &pks[0..pks.len()];
			let retval = unsafe {
				ffi::secp256k1_aggsig_build_scratch_and_verify(
					self.ctx,
					sig.as_ptr(),
					msg.as_ptr(),
					pks[0],
					pks.len(),
				)
			};
			match retval {
				0 => false,
				1 => true,
				_ => false,
			}
		} else {
			false
		}
	}
}

impl Drop for AggSigContext {
	fn drop(&mut self) {
		unsafe {
			ffi::secp256k1_aggsig_context_destroy(self.aggsig_ctx);
		}
	}
}

/*

#[cfg(test)]
mod tests {
	use super::{
		add_signatures_single, export_secnonce_single, sign_single, verify_single, verify_batch,
		AggSigContext, Secp256k1,
	};
	use crate::aggsig::subtract_partial_signature;
use crate::ffi;
	use crate::key::{PublicKey, SecretKey};
	use rand::{thread_rng, Rng};
	use crate::ContextFlag;
	use crate::{AggSigPartialSignature, Message, Signature};

	#[test]
	fn test_aggsig_multisig() {
		let numkeys = 5;
		let secp = Secp256k1::with_caps(ContextFlag::Full);
		let mut keypairs: Vec<(SecretKey, PublicKey)> = vec![];
		for _ in 0..numkeys {
			keypairs.push(secp.generate_keypair(&mut thread_rng()).unwrap());
		}
		let pks: Vec<PublicKey> = keypairs.clone().into_iter().map(|(_, p)| p).collect();
		println!(
			"Creating aggsig context with {} pubkeys: {:?}",
			pks.len(),
			pks
		);
		let aggsig = AggSigContext::new(&secp, &pks);
		println!("Generating nonces for each index");
		for i in 0..numkeys {
			let retval = aggsig.generate_nonce(i);
			println!("{} returned {}", i, retval);
			assert!(retval == true);
		}

		let mut msg = [0u8; 32];
		thread_rng().fill(&mut msg);
		let msg = Message::from_slice(&msg).unwrap();
		let mut partial_sigs: Vec<AggSigPartialSignature> = vec![];
		for i in 0..numkeys {
			println!(
				"Partial sign message: {:?} at index {}, SK:{:?}",
				msg, i, keypairs[i].0
			);

			let result = aggsig.partial_sign(msg, keypairs[i].0.clone(), i);
			match result {
				Ok(ps) => {
					println!("Partial sig: {:?}", ps);
					partial_sigs.push(ps);
				}
				Err(e) => panic!("Partial sig failed: {}", e),
			}
		}

		let result = aggsig.combine_signatures(&partial_sigs);

		let combined_sig = match result {
			Ok(cs) => {
				println!("Combined sig: {:?}", cs);
				cs
			}
			Err(e) => panic!("Combining partial sig failed: {}", e),
		};

		println!(
			"Verifying Combined sig: {:?}, msg: {:?}, pks:{:?}",
			combined_sig, msg, pks
		);
		let result = aggsig.verify(combined_sig, msg, &pks);
		println!("Signature verification: {}", result);
	}

	#[test]
	fn test_aggsig_single() {
		let secp = Secp256k1::with_caps(ContextFlag::Full);
		let (sk, pk) = secp.generate_keypair(&mut thread_rng()).unwrap();

		println!(
			"Performing aggsig single context with seckey, pubkey: {:?},{:?}",
			sk, pk
		);

		let mut msg = [0u8; 32];
		thread_rng().fill(&mut msg);
		let msg = Message::from_slice(&msg).unwrap();
		let sig = sign_single(&secp, &msg, &sk, None, None, None, None, None).unwrap();

		println!(
			"Verifying aggsig single: {:?}, msg: {:?}, pk:{:?}",
			sig, msg, pk
		);
		let result = verify_single(&secp, &sig, &msg, None, &pk, None, None, false);
		println!("Signature verification single (correct): {}", result);
		assert!(result == true);

		let mut msg = [0u8; 32];
		thread_rng().fill(&mut msg);
		let msg = Message::from_slice(&msg).unwrap();
		println!(
			"Verifying aggsig single: {:?}, msg: {:?}, pk:{:?}",
			sig, msg, pk
		);
		let result = verify_single(&secp, &sig, &msg, None, &pk, None, None, false);
		println!("Signature verification single (wrong message): {}", result);
		assert!(result == false);

		// test optional extra key
		let mut msg = [0u8; 32];
		thread_rng().fill(&mut msg);
		let msg = Message::from_slice(&msg).unwrap();
		let (sk_extra, pk_extra) = secp.generate_keypair(&mut thread_rng()).unwrap();
		let sig = sign_single(&secp, &msg, &sk, None, Some(&sk_extra), None, None, None).unwrap();
		let result = verify_single(&secp, &sig, &msg, None, &pk, None, Some(&pk_extra), false);
		assert!(result == true);
	}

	#[test]
	fn test_aggsig_batch() {
		let secp = Secp256k1::with_caps(ContextFlag::Full);

		let mut sigs: Vec<Signature> = vec![];
		let mut msgs: Vec<Message> = vec![];
		let mut pub_keys: Vec<PublicKey> = vec![];

		for _ in 0..100 {
			let (sk, pk) = secp.generate_keypair(&mut thread_rng()).unwrap();
			let mut msg = [0u8; 32];
			thread_rng().fill(&mut msg);

			let msg = Message::from_slice(&msg).unwrap();
			let sig = sign_single(&secp, &msg, &sk, None, None, None, Some(&pk), None).unwrap();

			let result_single = verify_single(&secp, &sig, &msg, None, &pk, Some(&pk), None, false);
			assert!(result_single == true);

			pub_keys.push(pk);
			msgs.push(msg);
			sigs.push(sig);
		}

		println!("Verifying aggsig batch of 100");
		let result = verify_batch(&secp, &sigs, &msgs, &pub_keys);
		assert!(result == true);
	}

	#[test]
	fn test_aggsig_fuzz() {
		let secp = Secp256k1::with_caps(ContextFlag::Full);
		let (sk, pk) = secp.generate_keypair(&mut thread_rng()).unwrap();

		println!(
			"Performing aggsig single context with seckey, pubkey: {:?},{:?}",
			sk, pk
		);

		let mut msg = [0u8; 32];
		thread_rng().fill(&mut msg);
		let msg = Message::from_slice(&msg).unwrap();
		let sig = sign_single(&secp, &msg, &sk, None, None, None, None, None).unwrap();

		// force sig[32..] as 0 to simulate Fuzz test
		let corrupted = &mut [0u8; 64];
		let mut i = 0;
		for elem in corrupted[..32].iter_mut() {
			*elem = sig.0[i];
			i += 1;
		}
		let corrupted_sig: Signature = Signature {
			0: ffi::Signature(*corrupted),
		};
		println!(
			"Verifying aggsig single: {:?}, msg: {:?}, pk:{:?}",
			corrupted_sig, msg, pk
		);
		let result = verify_single(&secp, &corrupted_sig, &msg, None, &pk, None, None, false);
		println!("Signature verification single (correct): {}", result);
		assert!(result == false);

		// force sig[0..32] as 0 to simulate Fuzz test
		let corrupted = &mut [0u8; 64];
		let mut i = 32;
		for elem in corrupted[32..].iter_mut() {
			*elem = sig.0[i];
			i += 1;
		}
		let corrupted_sig: Signature = Signature {
			0: ffi::Signature(*corrupted),
		};
		println!(
			"Verifying aggsig single: {:?}, msg: {:?}, pk:{:?}",
			corrupted_sig, msg, pk
		);
		let result = verify_single(&secp, &corrupted_sig, &msg, None, &pk, None, None, false);
		println!("Signature verification single (correct): {}", result);
		assert!(result == false);

		// force pk as 0 to simulate Fuzz test
		let zero_pk = PublicKey::new();
		println!(
			"Verifying aggsig single: {:?}, msg: {:?}, pk:{:?}",
			sig, msg, zero_pk
		);
		let result = verify_single(&secp, &sig, &msg, None, &zero_pk, None, None, false);
		println!("Signature verification single (correct): {}", result);
		assert!(result == false);

		let mut sigs: Vec<Signature> = vec![];
		sigs.push(sig);
		let mut msgs: Vec<Message> = vec![];
		msgs.push(msg);
		let mut pub_keys: Vec<PublicKey> = vec![];
		pub_keys.push(zero_pk);
		println!(
			"Verifying aggsig batch: {:?}, msg: {:?}, pk:{:?}",
			sig, msg, zero_pk
		);
		let result = verify_batch(&secp, &sigs, &msgs, &pub_keys);
		println!("Signature verification batch: {}", result);
		assert!(result == false);


		// force pk[0..32] as 0 to simulate Fuzz test
		let corrupted = &mut [0u8; 64];
		let mut i = 32;
		for elem in corrupted[32..].iter_mut() {
			*elem = pk.0[i];
			i += 1;
		}
		let corrupted_pk: PublicKey = PublicKey {
			0: ffi::PublicKey(*corrupted),
		};
		println!(
			"Verifying aggsig single: {:?}, msg: {:?}, pk:{:?}",
			sig, msg, corrupted_pk
		);
		let result = verify_single(&secp, &sig, &msg, None, &corrupted_pk, None, None, false);
		println!("Signature verification single (correct): {}", result);
		assert!(result == false);

		// more tests on other parameters
		let zero_pk = PublicKey::new();
		let result = verify_single(
			&secp,
			&sig,
			&msg,
			Some(&zero_pk),
			&zero_pk,
			Some(&zero_pk),
			Some(&zero_pk),
			false,
		);
		assert!(result == false);

		let mut msg = [0u8; 32];
		thread_rng().fill(&mut msg);
		let msg = Message::from_slice(&msg).unwrap();
		if sign_single(
			&secp,
			&msg,
			&sk,
			None,
			None,
			Some(&zero_pk),
			Some(&zero_pk),
			Some(&zero_pk),
		).is_ok()
		{
			panic!("sign_single should fail on zero public key, but not!");
		}
	}

	#[test]
	fn test_aggsig_exchange() {
		for _ in 0..20 {
			let secp = Secp256k1::with_caps(ContextFlag::Full);
			// Generate keys for sender, receiver
			let (sk1, pk1) = secp.generate_keypair(&mut thread_rng()).unwrap();
			let (sk2, pk2) = secp.generate_keypair(&mut thread_rng()).unwrap();

			// Generate nonces for sender, receiver
			let secnonce_1 = export_secnonce_single(&secp).unwrap();
			let secnonce_2 = export_secnonce_single(&secp).unwrap();

			// Calculate public nonces
			let _ = PublicKey::from_secret_key(&secp, &secnonce_1).unwrap();
			let pubnonce_2 = PublicKey::from_secret_key(&secp, &secnonce_2).unwrap();

			// And get the total
			let mut nonce_sum = pubnonce_2.clone();
			let _ = nonce_sum.add_exp_assign(&secp, &secnonce_1);

			// Random message
			let mut msg = [0u8; 32];
			thread_rng().fill(&mut msg);
			let msg = Message::from_slice(&msg).unwrap();

			// Add public keys (for storing in e)
			let mut pk_sum = pk2.clone();
			let _ = pk_sum.add_exp_assign(&secp, &sk1);

			// Receiver signs
			let sig1 = sign_single(
				&secp,
				&msg,
				&sk1,
				Some(&secnonce_1),
				None,
				Some(&nonce_sum),
				Some(&pk_sum),
				Some(&nonce_sum),
			).unwrap();

			// Sender verifies receivers sig
			let result = verify_single(
				&secp,
				&sig1,
				&msg,
				Some(&nonce_sum),
				&pk1,
				Some(&pk_sum),
				None,
				true,
			);
			assert!(result == true);

			// Sender signs
			let sig2 = sign_single(
				&secp,
				&msg,
				&sk2,
				Some(&secnonce_2),
				None,
				Some(&nonce_sum),
				Some(&pk_sum),
				Some(&nonce_sum),
			).unwrap();

			// Receiver verifies sender's sig
			let result = verify_single(
				&secp,
				&sig2,
				&msg,
				Some(&nonce_sum),
				&pk2,
				Some(&pk_sum),
				None,
				true,
			);
			assert!(result == true);

			let sig_vec = vec![&sig1, &sig2];
			// Receiver calculates final sig
			let final_sig = add_signatures_single(&secp, sig_vec, &nonce_sum).unwrap();

			// Verification of final sig:
			let result = verify_single(
				&secp,
				&final_sig,
				&msg,
				None,
				&pk_sum,
				Some(&pk_sum),
				None,
				false,
			);
			assert!(result == true);

			// Subtract sig1 from final sig
			let (res_sig, res_sig_opt) = subtract_partial_signature(&secp, &final_sig, &sig1).unwrap();
			assert!(res_sig == sig2 || res_sig_opt == Some(sig2));

			// Subtract sig2 from final sig for good measure
			let (res_sig, res_sig_opt) = subtract_partial_signature(&secp, &final_sig, &sig2).unwrap();
			assert!(res_sig == sig1 || res_sig_opt == Some(sig1));
		}
	}
}
*/
