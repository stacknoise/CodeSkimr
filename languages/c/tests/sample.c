#include <stdio.h>

/* A point. */
struct point {
    int x;
    int y;
};

enum color { RED, GREEN };

union value { int i; float f; };

typedef struct { int a; } pair;

static int *find(int *items, int n) {
    for (int i = 0; i < n; i++) {
        if (items[i] > 0 && i != 3) {
            return &items[i];
        }
    }
    return NULL;
}

int main(void) {
    return find(0, 0) ? 1 : 0;
}

int (*getcb(void))(int) {
    return 0;
}

char *(*getname(int n))(void) {
    return 0;
}
