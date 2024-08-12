/*
 * Represents a hashable, comparable array type.
 */

#pragma once
#include <algorithm>
#include <array>
#include <cstdint>
#include <cstring>

namespace apg {

template <typename T, int N>
class nicearray : public std::array<T, N> {
public:
    nicearray() = default;

    template <typename... U>
    nicearray(U... ts) : std::array<T, N>{ts...} {
    }

    uint64_t hash() {
        const static int k64 = (sizeof(T) * N) / 8;

        uint64_t hcopy[k64];
        std::memcpy(hcopy, this->data(), 8 * k64);

        uint64_t h = hcopy[0];
        for (int i = 1; i < k64; i++) {
            h -= (h << 7);
            h += hcopy[i];
        }

        return h;
    }

    bool iszero() const {
        return std::all_of(this->begin(), this->end(), [](T x) { return x == 0; });
    }
};

}  // namespace apg
