#define _GNU_SOURCE
#include <string.h>

/* Link an actual post-baseline glibc symbol on Ubuntu 24.04. */
int main(int argc, char **argv) {
    char destination[16];
    return strlcpy(destination, argv[argc - 1], sizeof(destination)) == 0;
}
