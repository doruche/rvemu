#include <stdio.h>
#include <stdlib.h>

#define N (1000 * 1000)

static int space[1024];

int primes[N] = {2};
size_t primes_count = 1;

int
is_prime(int x) {
    if (x < 2) return 0;
    for (int i = 2; i * i <= x; i++) {
        if (x % i == 0) return 0;
    }
    return 1;
}

int
main(void) {
    for (int i = 3; primes_count < N; i++) {
        if (is_prime(i)) {
            primes[primes_count++] = i;
        }
    }

    printf("%d\n", primes[primes_count - 1]);

    return 0;
}