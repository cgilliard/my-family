#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#ifdef __APPLE__
#include <sys/event.h>
#endif	// __APPLE__
#ifdef __linux__
#include <sys/epoll.h>
#endif	// __linux__
#include <sys/un.h>
#include <unistd.h>

#define MULTIPLEX_REGISTER_TYPE_NONE 0
#define MULTIPLEX_REGISTER_TYPE_FLAG_READ 0x1
#define MULTIPLEX_REGISTER_TYPE_FLAG_WRITE (0x1 << 1)

#define ERROR_SOCKET -1
#define ERROR_CONNECT -2
#define ERROR_SETSOCKOPT -3
#define ERROR_BIND -4
#define ERROR_LISTEN -5
#define ERROR_ACCEPT -6
#define ERROR_FCNTL -7
#define ERROR_REGISTER -8
#define ERROR_MULTIPLEX_INIT -9
#define ERROR_GETSOCKNAME -10
#define ERROR_EAGAIN -11

long long __fd_count = 0;

long long getfdcount() { return __fd_count; }

int close_impl(int fd) {
	int ret = close(fd);
	if (ret == 0) {
#ifdef TEST
		__atomic_fetch_sub(&__fd_count, 1, __ATOMIC_SEQ_CST);
#endif	// TEST
	}
	return ret;
}

typedef struct SocketHandle {
	int fd;
} SocketHandle;

typedef struct MultiplexHandle {
	int fd;
} MultiplexHandle;

unsigned long long int socket_handle_size() { return sizeof(SocketHandle); }
unsigned long long socket_event_size() {
#ifdef __APPLE__
	return sizeof(struct kevent);
#endif	// __APPLE__
#ifdef __linux__
	return sizeof(struct epoll_event);
#endif	// __linux__
}
unsigned long long socket_multiplex_handle_size() {
	return sizeof(MultiplexHandle);
}

_Bool socket_handle_eq(SocketHandle *h1, SocketHandle *h2) {
	return h1->fd == h2->fd;
}

int socket_connect(SocketHandle *s, unsigned char addr[4], int port) {
	s->fd = socket(AF_INET, SOCK_STREAM, 0);
	if (s->fd < 0) return ERROR_SOCKET;
#ifdef TEST
	__atomic_fetch_add(&__fd_count, 1, __ATOMIC_SEQ_CST);
#endif	// TEST

	struct sockaddr_in serv_addr;
	memset(&serv_addr, 0, sizeof(serv_addr));
	serv_addr.sin_family = AF_INET;
	serv_addr.sin_port = htons(port);
	memcpy(&serv_addr.sin_addr.s_addr, addr, 4);

	if (connect(s->fd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) <
	    0) {
		perror("connect");
		close_impl(s->fd);
		return ERROR_CONNECT;
	}

	int flags = fcntl(s->fd, F_GETFL, 0);
	if (flags < 0) {
		close_impl(s->fd);
		return ERROR_FCNTL;
	}

	if (fcntl(s->fd, F_SETFL, flags | O_NONBLOCK) < 0) {
		close_impl(s->fd);
		return ERROR_FCNTL;
	}

	return 0;
}

int socket_clear_pipe(SocketHandle *s) {
	int capacity = 512;
	char buf[capacity];
	while (1) {
		int ret = read(s->fd, buf, capacity);
		if (ret <= 0) {
			if (errno == EAGAIN) {
				return ERROR_EAGAIN;
			} else {
				return -1;
			}
		}
	}
}

int open_pipe(int *handles) {
	int ret = pipe((int *)handles);
	if (ret == 0) {
#ifdef TEST
		__atomic_fetch_add(&__fd_count, 2, __ATOMIC_SEQ_CST);
#endif	// TEST
		int flags = fcntl(handles[0], F_GETFL, 0);
		if (flags == -1) {
			perror("fcntl");
			close_impl(handles[0]);
			close_impl(handles[1]);
			return -1;
		}

		flags |= O_NONBLOCK;
		if (fcntl(handles[0], F_SETFL, flags) == -1) {
			perror("fcntl");
			close_impl(handles[0]);
			close_impl(handles[1]);
			return -1;
		}

		flags = fcntl(handles[1], F_GETFL, 0);
		if (flags == -1) {
			perror("fcntl");
			close_impl(handles[0]);
			close_impl(handles[1]);
			return -1;
		}

		flags |= O_NONBLOCK;
		if (fcntl(handles[1], F_SETFL, flags) == -1) {
			perror("fcntl");
			close_impl(handles[0]);
			close_impl(handles[1]);
			return -1;
		}
	}

	return ret;
}

int socket_shutdown(SocketHandle *s) { return shutdown(s->fd, SHUT_RDWR); }
int socket_close(SocketHandle *s) { return close_impl(s->fd); }
int socket_listen(SocketHandle *s, unsigned char addr[4], int port,
		  int backlog) {
	int opt = 1;
	struct sockaddr_in address;

	s->fd = socket(AF_INET, SOCK_STREAM, 0);
#ifdef TEST
	__atomic_fetch_add(&__fd_count, 1, __ATOMIC_SEQ_CST);
#endif	// TEST
	if (s->fd < 0) return ERROR_SOCKET;
	if (setsockopt(s->fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt))) {
		close_impl(s->fd);
		return ERROR_SETSOCKOPT;
	}

	if (setsockopt(s->fd, SOL_SOCKET, SO_REUSEPORT, &opt, sizeof(opt))) {
		close_impl(s->fd);
		return ERROR_SETSOCKOPT;
	}
	int flags = fcntl(s->fd, F_GETFL, 0);
	if (flags < 0) {
		close_impl(s->fd);
		return ERROR_FCNTL;
	}

	if (fcntl(s->fd, F_SETFL, flags | O_NONBLOCK) < 0) {
		close_impl(s->fd);
		return ERROR_FCNTL;
	}

	address.sin_family = AF_INET;
	address.sin_addr.s_addr = INADDR_ANY;
	address.sin_port = htons(port);

	if (bind(s->fd, (struct sockaddr *)&address, sizeof(address)) < 0) {
		close_impl(s->fd);
		return ERROR_BIND;
	}

	if (listen(s->fd, backlog) < 0) {
		close_impl(s->fd);
		return ERROR_LISTEN;
	}

	socklen_t addr_len = sizeof(address);
	if (getsockname(s->fd, (struct sockaddr *)&address, &addr_len) < 0) {
		close_impl(s->fd);
		return ERROR_GETSOCKNAME;
	}
	port = ntohs(address.sin_port);
	return port;
}

int socket_accept(SocketHandle *s, SocketHandle *accepted) {
	struct sockaddr_in client_addr;
	socklen_t client_len = sizeof(client_addr);
	accepted->fd =
	    accept(s->fd, (struct sockaddr *)&client_addr, &client_len);
	if (accepted->fd < 0) {
		if (errno == EAGAIN) {
			return ERROR_EAGAIN;
		}
		return ERROR_ACCEPT;
	}

#ifdef TEST
	__atomic_fetch_add(&__fd_count, 1, __ATOMIC_SEQ_CST);
#endif	// TEST

	int flags = fcntl(accepted->fd, F_GETFL, 0);

	if (fcntl(accepted->fd, F_SETFL, flags | O_NONBLOCK) < 0) {
		close_impl(accepted->fd);
		return ERROR_FCNTL;
	}

	return 0;
}

long long socket_send(SocketHandle *s, const char *buf,
		      unsigned long long len) {
	long long ret = write(s->fd, buf, len);
	if (ret < 0) {
		if (errno == EAGAIN) {
			return ERROR_EAGAIN;
		}
	}
	return ret;
}

long long socket_recv(SocketHandle *s, char *buf, unsigned long long capacity) {
	int ret = read(s->fd, buf, capacity);
	if (ret < 0) {
		if (errno == EAGAIN) {
			return ERROR_EAGAIN;
		}
	}
	return ret;
}
int socket_multiplex_init(MultiplexHandle *multiplex) {
#ifdef __APPLE__
	multiplex->fd = kqueue();
#endif	// __APPLE__
#ifdef __linux__
	multiplex->fd = epoll_create1(0);
#endif	// __linux__
	if (multiplex->fd < 0) return ERROR_MULTIPLEX_INIT;

#ifdef TEST
	__atomic_fetch_add(&__fd_count, 1, __ATOMIC_SEQ_CST);
#endif	// TEST
	return 0;
}
#ifdef __APPLE__
int socket_multiplex_register(MultiplexHandle *multiplex, SocketHandle *s,
			      int flags, void *ptr) {
	struct kevent change_event[2];

	int event_count = 0;

	if (flags & MULTIPLEX_REGISTER_TYPE_FLAG_READ) {
		EV_SET(&change_event[event_count], s->fd, EVFILT_READ,
		       EV_ADD | EV_ENABLE | EV_CLEAR, 0, 0, ptr);
		event_count++;
	}

	if (flags & MULTIPLEX_REGISTER_TYPE_FLAG_WRITE) {
		EV_SET(&change_event[event_count], s->fd, EVFILT_WRITE,
		       EV_ADD | EV_ENABLE | EV_CLEAR, 0, 0, ptr);
		event_count++;
	}

	if (kevent(multiplex->fd, change_event, event_count, NULL, 0, NULL) <
	    0) {
		return ERROR_REGISTER;
	}
	return 0;
}
#endif	// __APPLE__
#ifdef __linux__
int socket_multiplex_register(MultiplexHandle *multiplex, SocketHandle *s,
			      int flags, void *ptr) {
	struct epoll_event ev;
	int event_flags = 0;

	if (flags & MULTIPLEX_REGISTER_TYPE_FLAG_READ) {
		event_flags |= EPOLLIN;
	}

	if (flags & MULTIPLEX_REGISTER_TYPE_FLAG_WRITE) {
		event_flags |= EPOLLOUT;
	}

	ev.events = event_flags;
	if (ptr == NULL)
		ev.data.fd = s->fd;
	else
		ev.data.ptr = ptr;

	if (epoll_ctl(multiplex->fd, EPOLL_CTL_ADD, s->fd, &ev) < 0) {
		if (errno == EEXIST) {
			if (epoll_ctl(multiplex->fd, EPOLL_CTL_MOD, s->fd,
				      &ev) < 0) {
				return ERROR_REGISTER;
			}
		} else
			return ERROR_REGISTER;
	}

	return 0;
}
#endif	// __linux__
#ifdef __APPLE__
int socket_multiplex_unregister_write(MultiplexHandle *multiplex,
				      SocketHandle *s, void *ptr) {
	struct kevent change_event[1];
	int event_count = 1;

	EV_SET(&change_event[0], s->fd, EVFILT_WRITE,
	       EV_DELETE | EV_ENABLE | EV_CLEAR, 0, 0, NULL);

	if (kevent(multiplex->fd, change_event, event_count, NULL, 0, NULL) <
	    0) {
		return ERROR_REGISTER;
	}
	return 0;
}
#endif	// __APPLE__
#ifdef __linux__
int socket_multiplex_unregister_write(MultiplexHandle *multiplex,
				      SocketHandle *s, void *ptr) {
	struct epoll_event event;
	event.data.ptr = ptr;
	event.events = EPOLLIN;

	if (epoll_ctl(multiplex->fd, EPOLL_CTL_MOD, s->fd, &event) < 0)
		return ERROR_REGISTER;

	return 0;
}
#endif	// __linux__

/*
int socket_multiplex_wait(MultiplexHandle *multiplex, void *events,
			  int max_events, long long timeout_millis) {
#ifdef __APPLE__
	return kevent(multiplex->fd, NULL, 0, (struct kevent *)events,
		      max_events, NULL);
#endif	// __APPLE__
#ifdef __linux__
	return epoll_wait(multiplex->fd, (struct epoll_event *)events,
			  max_events, -1);
#endif	// __linux__
}
*/

int socket_multiplex_wait(MultiplexHandle *multiplex, void *events,
			  int max_events, long long timeout_millis) {
#ifdef __APPLE__
	struct timespec ts;
	struct timespec *timeout_ptr = NULL;

	if (timeout_millis >= 0) {
		ts.tv_sec = timeout_millis / 1000;
		ts.tv_nsec = (timeout_millis % 1000) * 1000000;
		timeout_ptr = &ts;
	}

	return kevent(multiplex->fd, NULL, 0, (struct kevent *)events,
		      max_events, timeout_ptr);
#endif	// __APPLE__

#ifdef __linux__
	int timeout = (timeout_millis >= 0) ? (int)timeout_millis : -1;

	return epoll_wait(multiplex->fd, (struct epoll_event *)events,
			  max_events, timeout);
#endif	// __linux__
}

int socket_fd(SocketHandle *s) { return s->fd; }

void *socket_event_ptr(void *event) {
#ifdef __APPLE__
	struct kevent *kv = (struct kevent *)event;
	return kv->udata;
#elif defined(__linux__)
	struct epoll_event *epoll_ev = (struct epoll_event *)event;
	return epoll_ev->data.ptr;
#else
	return NULL;
#endif
}

void socket_event_handle(SocketHandle *s, void *event) {
#ifdef __APPLE__
	struct kevent *kv = event;
	s->fd = kv->ident;
#endif	// __APPLE__
#ifdef __linux__
	struct epoll_event *epoll_ev = event;
	s->fd = epoll_ev->data.fd;
#endif	// __linux__
}

_Bool socket_event_is_read(void *event) {
#ifdef __APPLE__
	struct kevent *kv = event;
	return kv->filter == EVFILT_READ;
#endif	// __APPLE__
#ifdef __linux__
	struct epoll_event *epoll_ev = event;
	return epoll_ev->events & EPOLLIN;
#endif	// __linux__
}

_Bool socket_event_is_write(void *event) {
#ifdef __APPLE__
	struct kevent *kv = event;
	return kv->filter == EVFILT_WRITE;
#endif	// __APPLE__
#ifdef __linux__
	struct epoll_event *epoll_ev = event;
	return epoll_ev->events & EPOLLOUT;
#endif	// __linux__
}
