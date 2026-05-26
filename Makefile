CC ?= cc
CFLAGS ?= -std=c11 -Wall -Wextra -pedantic -pthread -O2
CPPFLAGS ?= -Iinclude
LDFLAGS ?= -pthread

TARGET := cowtop
SOURCES := src/main.c src/proc_reader.c
HEADERS := include/proc_reader.h

.PHONY: all clean run

all: $(TARGET)

$(TARGET): $(SOURCES) $(HEADERS)
	$(CC) $(CPPFLAGS) $(CFLAGS) $(SOURCES) $(LDFLAGS) -o $@

run: $(TARGET)
	./$(TARGET) $(ARGS)

clean:
	rm -f $(TARGET) report.txt
