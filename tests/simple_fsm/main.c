#include <arpa/inet.h>
#include <errno.h>
#include <netdb.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

static volatile sig_atomic_t keep_running = 1;

static void handle_signal(int sig) {
  (void)sig;
  keep_running = 0;
}

typedef struct {
  const char *states[5];
  int idx;
} fsm_t;

static void fsm_init(fsm_t *f) {
  f->states[0] = "A";
  f->states[1] = "B";
  f->states[2] = "C";
  f->states[3] = "D";
  f->states[4] = "E";
  f->idx = 0;
}

static const char *fsm_state(fsm_t *f) { return f->states[f->idx]; }

static const char *fsm_next(fsm_t *f) {
  f->idx = (f->idx + 1) % 5;
  return f->states[f->idx];
}

static void send_response(int fd, int status, const char *body) {
  char header[256];
  int body_len = (int)strlen(body);
  int n = snprintf(header, sizeof(header),
                   "HTTP/1.1 %d OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: %d\r\nConnection: close\r\n\r\n",
                   status, body_len);
  if (n < 0) return;
  (void)write(fd, header, (size_t)n);
  (void)write(fd, body, (size_t)body_len);
}

static void send_405(int fd) {
  const char *body = "Error: method not allowed";
  char header[256];
  int n = snprintf(header, sizeof(header),
                   "HTTP/1.1 405 Method Not Allowed\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: %zu\r\nConnection: close\r\n\r\n",
                   strlen(body));
  if (n < 0) return;
  (void)write(fd, header, (size_t)n);
  (void)write(fd, body, strlen(body));
}

static void handle_conn(int cfd, fsm_t *f) {
  char buf[2048];
  ssize_t r = read(cfd, buf, sizeof(buf) - 1);
  if (r <= 0) return;
  buf[r] = '\0';

  char method[8] = {0};
  char path[64] = {0};
  if (sscanf(buf, "%7s %63s", method, path) != 2) return;

  if (strcmp(path, "/state") == 0) {
    if (strcmp(method, "GET") != 0) {
      send_405(cfd);
      return;
    }
    char body[8];
    snprintf(body, sizeof(body), "\"%s\"", fsm_state(f));
    send_response(cfd, 200, body);
    return;
  }

  if (strcmp(path, "/next") == 0) {
    if (strcmp(method, "POST") != 0) {
      send_405(cfd);
      return;
    }
    char body[8];
    snprintf(body, sizeof(body), "\"%s\"", fsm_next(f));
    send_response(cfd, 200, body);
    return;
  }

  const char *body = "Error: bad request";
  send_response(cfd, 400, body);
}

int main(int argc, char **argv) {
  (void)argc;
  (void)argv;

  signal(SIGINT, handle_signal);
  signal(SIGTERM, handle_signal);

  int port = 8080;
  int sfd = socket(AF_INET, SOCK_STREAM, 0);
  if (sfd < 0) {
    perror("socket");
    return 1;
  }
  int yes = 1;
  setsockopt(sfd, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));

  struct sockaddr_in addr;
  memset(&addr, 0, sizeof(addr));
  addr.sin_family = AF_INET;
  addr.sin_addr.s_addr = htonl(INADDR_ANY);
  addr.sin_port = htons((uint16_t)port);

  if (bind(sfd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
    perror("bind");
    close(sfd);
    return 1;
  }

  if (listen(sfd, 64) < 0) {
    perror("listen");
    close(sfd);
    return 1;
  }

  fsm_t f;
  fsm_init(&f);

  while (keep_running) {
    struct sockaddr_in caddr;
    socklen_t clen = sizeof(caddr);
    int cfd = accept(sfd, (struct sockaddr *)&caddr, &clen);
    if (cfd < 0) {
      if (errno == EINTR) continue;
      perror("accept");
      break;
    }
    handle_conn(cfd, &f);
    close(cfd);
  }

  close(sfd);
  return 0;
}


