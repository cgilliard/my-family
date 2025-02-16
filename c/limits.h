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

#ifndef _LIMITS_H__
#define _LIMITS_H__

#ifndef INT64_MAX
#define INT64_MAX ((long long)0x7FFFFFFFFFFFFFFFLL)
#endif

#ifndef INT64_MIN
#define INT64_MIN (-INT64_MAX - 1)
#endif

#ifndef INT32_MAX
#define INT32_MAX ((int)0x7FFFFFFF)
#endif

#ifndef INT32_MIN
#define INT32_MIN (-INT32_MAX - 1)
#endif

#ifndef UINT64_MAX
#define UINT64_MAX ((unsigned int)0xFFFFFFFFFFFFFFFFULL)
#endif

#ifndef UINT32_MAX
#define UINT32_MAX ((unsigned int)0xFFFFFFFFULL)
#endif

#endif	// _LIMITS_H__
