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

#ifndef _BASE_CPSRNG__
#define _BASE_CPSRNG__

#include "aes.h"
#include "types.h"

typedef struct CsprngCtx {
	struct AES_ctx ctx;
} CsprngCtx;

void cpsrng_reseed();
void cpsrng_rand_byte(byte *v);
void cpsrng_rand_i64(int64 *v);
void cpsrng_rand_int(int *v);
void cpsrng_rand_bytes(void *v, unsigned long long size);

CsprngCtx *cpsrng_context_create();
void cpsrng_context_destroy(CsprngCtx *);
void cpsrng_rand_bytes_ctx(CsprngCtx *, void *v, unsigned long long size);

#ifdef TEST
void cpsrng_test_seed(byte iv[16], byte key[32]);
#endif	// TEST

#endif	// _BASE_CPSRNG__
