CC ?= clang

TARGETS_FILE ?=

CFLAGS = -Wall -Wextra

SRCS = main.c src/math_utils.c

OBJS = $(SRCS:.c=.o)

TARGET = test_program

all: $(TARGET)

$(TARGET): $(OBJS) FORCE
	$(CC) $(CFLAGS) -o $@ $(OBJS)

%.o: %.c FORCE
	TARGETS_FILE=$(TARGETS_FILE) $(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(TARGET) $(OBJS)

FORCE:

.PHONY: all clean FORCE 
