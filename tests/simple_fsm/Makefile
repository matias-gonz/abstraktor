CC ?= clang
TARGETS_FILE ?=

CFLAGS ?= -Wall -Wextra -O2
LDFLAGS ?=

TARGET = fsm_app
SRCS = main.c
OBJS = $(SRCS:.c=.o)

all: $(TARGET)


$(TARGET): $(OBJS) FORCE
	$(CC) $(CFLAGS) -o $@ $(OBJS) $(LDFLAGS)

%.o: %.c FORCE
	TARGETS_FILE=$(TARGETS_FILE) $(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(TARGET) $(OBJS)

FORCE:

.PHONY: all clean FORCE

