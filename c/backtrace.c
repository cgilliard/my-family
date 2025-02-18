#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef __APPLE__
#include <dlfcn.h>
#include <mach/mach.h>
#endif	// __APPLE__

#include <util.h>

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
	cstring_cat_n(ret, NULL, 0);
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
			u64 address = cstring_strtoull(addr, 16);
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
						cstring_cat_n(ret, buffer,
							      strlen(buffer));
						i = bt->size;
						break;
					}
					cstring_cat_n(ret, buffer,
						      strlen(buffer));
				} else if (cstring_is_alpha_numeric(buffer)) {
					if (len && buffer[len - 1] == '\n') {
						len--;
						buffer[len] = ' ';
					}

					len_sum += len;
					if (len_sum >= 4 * PAGE_SIZE) break;
					cstring_cat_n(ret, buffer,
						      strlen(buffer));
					if (!cstring_compare(buffer, "main ")) {
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
				cstring_cat_n(ret, buffer, cstring_len(buffer));
			}
		}
		pclose(fp);
#else
		printf("WARN: Unsupported OS: cannot build backtraces!\n");
#endif
	}

	return ret;
}
