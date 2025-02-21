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

#ifndef _BASE_TYPES__
#define _BASE_TYPES__

#include "limits.h"

// primitives
typedef signed long long int64;
typedef unsigned char byte;
#define float64 double

// booleans
#define bool _Bool
#define true (_Bool)1
#define false (_Bool)0

// NULL
#ifndef NULL
#define NULL ((void *)0)
#endif	// NULL

#endif	// _BASE_TYPES__
