/*
 * Modular exponentiation and primality testing for 64-bit integers.
 * Adam P. Goucher, 2016
 */

#pragma once

#include <cstdint>

namespace apg {

inline bool isprime64(uint64_t p) {
    if (p < 2) {
        return false;
    }
    if (p < 4) {
        return true;
    }
    if (p % 2 == 0 || p % 3 == 0) {
        return false;
    }
    for (uint64_t i = 5; i * i <= p; i += 6) {
        if (p % i == 0 || p % (i + 2) == 0) {
            return false;
        }
    }
    return true;
}

inline uint64_t nextprime(uint64_t n) {
    uint64_t p = n + 1 + (n % 2);
    while (!isprime64(p)) {
        p += 2;
    }
    return p;
}

}  // namespace apg
