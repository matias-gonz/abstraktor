CC ?= clang

TARGETS_FILE ?=

CFLAGS = -Wall -Wextra

SRCS = main.c src/math_utils.c

OBJS = $(SRCS:.c=.o)

TARGET = test_program

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^

%.o: %.c
	TARGETS_FILE=$(TARGETS_FILE) $(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(TARGET) $(OBJS)

.PHONY: all clean 