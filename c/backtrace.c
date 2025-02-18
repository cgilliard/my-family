#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef __APPLE__
#include <dlfcn.h>
#include <mach/mach.h>
#endif	// __APPLE__

extern long long __alloc_count;
int backtrace(void **array, int capacity);
char **backtrace_symbols(void **array, int capacity);

#define u64 unsigned long long
#define MAX_BACKTRACE_ENTRIES 128
#define MAX_BACKTRACE_LEN (1024 * 1024)

int getpagesize();
#ifndef PAGE_SIZE
#define PAGE_SIZE (getpagesize())
#endif	// PAGE_SIZE

unsigned long long bt_cstring_len(const char *X) {
	const char *Y = X;
	while (*X) X++;
	return X - Y;
}

void bt_cstring_cat_n(char *X, char *Y, unsigned long long n) {
	X += bt_cstring_len(X);
	while (n-- && *Y) {
		*X = *Y;
		X++;
		Y++;
	}
	*X = 0;
}

int bt_cstring_char_is_alpha_numeric(char ch) {
	if (ch >= 'a' && ch <= 'z') return 1;
	if (ch >= 'A' && ch <= 'Z') return 1;
	if (ch >= '0' && ch <= '9') return 1;
	if (ch == '_' || ch == '\n') return 1;
	return 0;
}
int bt_cstring_is_alpha_numeric(const char *X) {
	if (*X >= '0' && *X <= '9') return 0;
	while (*X)
		if (!bt_cstring_char_is_alpha_numeric(*X++)) return 0;

	return 1;
}

unsigned long long bt_cstring_strtoull(const char *X, int base) {
	unsigned long long ret = 0, mul = 1, len = bt_cstring_len(X);
	while (len-- && X[len] != 'x') {
		ret += X[len] > '9' ? ((X[len] - 'a') + 10) * mul
				    : (X[len] - '0') * mul;
		mul *= base;
	}
	return ret;
}

int bt_cstring_compare(const char *X, const char *Y) {
	while (*X == *Y && *X) {
		X++;
		Y++;
	}
	if (*X > *Y) return 1;
	if (*Y > *X) return -1;
	return 0;
}

const char *backtrace_full(const char *binary, u64 len) {
	char *binary_null_term = malloc(len + 1);
	strncpy(binary_null_term, binary, len);
	binary_null_term[len] = 0;
	char *v = getenv("RUST_BACKTRACE");
	if (v == NULL || bt_cstring_len(v) == 0) {
		free(binary_null_term);
		return NULL;
	}
	void *array[MAX_BACKTRACE_ENTRIES];
	int size = backtrace(array, MAX_BACKTRACE_ENTRIES);
	char **strings = backtrace_symbols(array, size);
	char *ret = malloc(MAX_BACKTRACE_LEN);
	if (ret == NULL) {
		free(binary_null_term);
		return NULL;
	}
	bool term = false;
	int len_sum = 0;
	for (int i = 0; i < size; i++) {
		char address[256];
#ifdef __linux__
		int len = strlen(strings[i]);
		int last_plus = -1;

		while (len > 0) {
			if (strings[i][len] == '+') {
				last_plus = len;
				break;
			}
			len--;
		}
		if (last_plus > 0) {
			char *addr = strings[i] + last_plus + 1;
			int itt = 0;
			while (addr[itt]) {
				if (addr[itt] == ')') {
					addr[itt] = 0;
					break;
				}
				itt++;
			}
			u64 address = bt_cstring_strtoull(addr, 16);
			address -= 8;

			char command[256];
			snprintf(command, sizeof(command),
				 "addr2line -f -e %s %llx", binary_null_term,
				 address);

			void *fp = popen(command, "r");
			char buffer[128];
			while (fgets(buffer, sizeof(buffer), fp) != NULL) {
				int len = strlen(buffer);
				if (strstr(buffer, ".rs:")) {
					len_sum += len;
					if (len_sum >= 4 * PAGE_SIZE) break;
					if (term) {
						if (buffer[len - 1] == '\n')
							buffer[len - 1] = 0;
						bt_cstring_cat_n(
						    ret, buffer,
						    strlen(buffer));
						i = size;
						break;
					}
					bt_cstring_cat_n(ret, buffer,
							 strlen(buffer));
				} else if (bt_cstring_is_alpha_numeric(
					       buffer)) {
					if (len && buffer[len - 1] == '\n') {
						len--;
						buffer[len] = ' ';
					}

					len_sum += len;
					if (len_sum >= 4 * PAGE_SIZE) break;
					bt_cstring_cat_n(ret, buffer,
							 strlen(buffer));
					if (!bt_cstring_compare(buffer,
								"main ")) {
						term = true;
					}
				}
			}

			pclose(fp);
		}
#elif defined(__APPLE__)
		Dl_info info;
		dladdr(array[i], &info);
		u64 addr = 0x0000000100000000 + info.dli_saddr - info.dli_fbase;
		u64 offset = (u64)array[i] - (u64)info.dli_saddr;
		addr += offset;
		addr -= 4;
		snprintf(address, sizeof(address), "0x%llx", addr);
		char command[256];
		snprintf(command, sizeof(command),
			 "atos -fullPath -o %s -l 0x100000000 %s",
			 binary_null_term, address);
		void *fp = popen(command, "r");
		char buffer[128];

		while (fgets(buffer, sizeof(buffer), fp) != NULL) {
			int len = strlen(buffer);
			len_sum += len;
			if (len_sum >= MAX_BACKTRACE_LEN) break;
			if (strstr(buffer, "backtrace_full ") != buffer) {
				bt_cstring_cat_n(ret, buffer,
						 bt_cstring_len(buffer));
			}
		}
		pclose(fp);
#else
		printf("WARN: Unsupported OS: cannot build backtraces!\n");
#endif
	}

	if (strings && size) free(strings);
	free(binary_null_term);
	return ret;
}

typedef struct Backtrace {
	void **array;
	int size;
} Backtrace;

int backtrace_ptr(Backtrace *bt, int max_size) {
	if (!bt || max_size <= 0) {
		bt->size = 0;
		return 0;
	}

	if (getenv("RUST_BACKTRACE") == NULL) {
		bt->size = 0;
		return 0;
	}

	void **array = malloc(max_size * sizeof(void *));
	if (!array) {
		bt->size = 0;
		return 0;
	}
	int size = backtrace(array, max_size);
	bt->size = size;
	bt->array = array;
	return size;
}

int backtrace_size() { return sizeof(Backtrace); }

void backtrace_free(Backtrace *bt) {
	if (bt && bt->array) {
		free(bt->array);
		bt->array = NULL;
	}
}

char *backtrace_to_string(Backtrace *bt, char *binary) {
	bool term = false;
	char *ret = malloc(MAX_BACKTRACE_LEN);
	bt_cstring_cat_n(ret, NULL, 0);
	if (ret == NULL) return NULL;
	int len_sum = 0;

	char **strings = backtrace_symbols(bt->array, bt->size);

	for (int i = 0; i < bt->size; i++) {
		char address[256];
#ifdef __linux__
		int len = strlen(strings[i]);
		int last_plus = -1;

		while (len > 0) {
			if (strings[i][len] == '+') {
				last_plus = len;
				break;
			}
			len--;
		}
		if (last_plus > 0) {
			char *addr = strings[i] + last_plus + 1;
			int itt = 0;
			while (addr[itt]) {
				if (addr[itt] == ')') {
					addr[itt] = 0;
					break;
				}
				itt++;
			}
			u64 address = bt_cstring_strtoull(addr, 16);
			address -= 8;

			char command[256];
			snprintf(command, sizeof(command),
				 "addr2line -f -e %s %llx", binary, address);

			void *fp = popen(command, "r");
			char buffer[128];
			while (fgets(buffer, sizeof(buffer), fp) != NULL) {
				int len = strlen(buffer);
				if (strstr(buffer, ".rs:")) {
					len_sum += len;
					if (len_sum >= 4 * PAGE_SIZE) break;
					if (term) {
						if (buffer[len - 1] == '\n')
							buffer[len - 1] = 0;
						bt_cstring_cat_n(
						    ret, buffer,
						    strlen(buffer));
						i = bt->size;
						break;
					}
					bt_cstring_cat_n(ret, buffer,
							 strlen(buffer));
				} else if (bt_cstring_is_alpha_numeric(
					       buffer)) {
					if (len && buffer[len - 1] == '\n') {
						len--;
						buffer[len] = ' ';
					}

					len_sum += len;
					if (len_sum >= 4 * PAGE_SIZE) break;
					bt_cstring_cat_n(ret, buffer,
							 strlen(buffer));
					if (!bt_cstring_compare(buffer,
								"main ")) {
						term = true;
					}
				}
			}

			pclose(fp);
		}
#elif defined(__APPLE__)
		Dl_info info;
		dladdr(bt->array[i], &info);
		u64 addr = 0x0000000100000000 + info.dli_saddr - info.dli_fbase;
		u64 offset = (u64)bt->array[i] - (u64)info.dli_saddr;
		addr += offset;
		addr -= 4;
		snprintf(address, sizeof(address), "0x%llx", addr);
		char command[256];
		snprintf(command, sizeof(command),
			 "atos -fullPath -o %s -l 0x100000000 %s", binary,
			 address);
		void *fp = popen(command, "r");
		char buffer[128];

		while (fgets(buffer, sizeof(buffer), fp) != NULL) {
			int len = strlen(buffer);
			len_sum += len;
			if (len_sum >= MAX_BACKTRACE_LEN) break;
			if (strstr(buffer, "backtrace_full ") != buffer) {
				bt_cstring_cat_n(ret, buffer,
						 bt_cstring_len(buffer));
			}
		}
		pclose(fp);
#else
		printf("WARN: Unsupported OS: cannot build backtraces!\n");
#endif
	}

	return ret;
}
