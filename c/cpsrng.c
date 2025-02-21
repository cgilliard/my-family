// Copyright (c) 2024, The MyFamily Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include "cpsrng.h"

#include "aes.h"

void _exit(int);
int printf(const char *, ...);
int rand_bytes(unsigned char *buf, unsigned long long length);
void *alloc(unsigned long size);
void release(void *);

CsprngCtx *cpsrng_context_create() {
	CsprngCtx *ret = alloc(sizeof(CsprngCtx));
	if (ret) {
		byte iv[16];
		byte key[32];
		if (rand_bytes(key, 32)) {
			release(ret);
			return NULL;
		}
		if (rand_bytes(iv, 16)) {
			release(ret);
			return NULL;
		}

		AES_init_ctx_iv(&ret->ctx, key, iv);
	}
	return ret;
}
void cpsrng_context_destroy(CsprngCtx *ctx) { release(ctx); }
void cpsrng_rand_bytes_ctx(CsprngCtx *ctx, void *v, unsigned long long size) {
	AES_CTR_xcrypt_buffer(&ctx->ctx, (byte *)v, size);
}

static struct AES_ctx aes_ctx;

void cpsrng_reseed() {
	byte iv[16];
	byte key[32];
	if (rand_bytes(key, 32)) {
		printf("Could not generate entropy for AES key generation\n");
		_exit(-1);
	}
	if (rand_bytes(iv, 16)) {
		printf("Could not generate entropy for AES iv generation\n");
		_exit(-1);
	}

	AES_init_ctx_iv(&aes_ctx, key, iv);
}

// __attribute__ ((constructor)) guaranteed to be called before main.
// This will either succeed or exit before main is called.
void __attribute__((constructor)) __init_cpsrng() { cpsrng_reseed(); }

// note: not thread safe as user must ensure thread safety. This allows for
// flexible usage in a single thread, no locking is needed. In multi-threaded
// environments, locking may be used.
void cpsrng_rand_byte(byte *v) {
	AES_CTR_xcrypt_buffer(&aes_ctx, (byte *)v, sizeof(byte));
}

// note: not thread safe as user must ensure thread safety. This allows for
// flexible usage in a single thread, no locking is needed. In multi-threaded
// environments, locking may be used.
void cpsrng_rand_i64(int64 *v) {
	AES_CTR_xcrypt_buffer(&aes_ctx, (byte *)v, sizeof(int64));
}

// note: not thread safe as user must ensure thread safety. This allows for
// flexible usage in a single thread, no locking is needed. In multi-threaded
// environments, locking may be used.
void cpsrng_rand_int(int *v) {
	AES_CTR_xcrypt_buffer(&aes_ctx, (byte *)v, sizeof(int));
}

// note: not thread safe as user must ensure thread safety. This allows for
// flexible usage in a single thread, no locking is needed. In multi-threaded
// environments, locking may be used.
void cpsrng_rand_bytes(void *v, unsigned long long size) {
	AES_CTR_xcrypt_buffer(&aes_ctx, (byte *)v, size);
}

// only available in test mode for tests. Not used in production environments.
#ifdef TEST
void cpsrng_test_seed(byte iv[16], byte key[32]) {
	AES_init_ctx_iv(&aes_ctx, key, iv);
	int64 v0 = 0;
	cpsrng_rand_i64(&v0);
}
#endif	// TEST
