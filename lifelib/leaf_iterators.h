#pragma once

#include <cstdint>
#include <cstring>
#include <utility>

// The first 32 bytes are arguments to (V)PSHUFB; the remaining
// 32 bytes are arguments to VPERMD. We align on a 64-byte boundary
// for both (16-byte) SIMD necessity and to avoid cache misses.
const static uint8_t __lifeperm[] __attribute__((aligned(64))) = {
    0, 4, 8, 12, 2, 6, 10, 14, 1, 5, 9, 13, 3, 7, 11, 15, 0, 4, 8, 12, 2, 6, 10, 14, 1, 5, 9, 13, 3, 7, 11, 15,
    0, 0, 0, 0,  4, 0, 0,  0,  2, 0, 0, 0,  6, 0, 0,  0,  1, 0, 0, 0,  5, 0, 0,  0,  3, 0, 0, 0,  7, 0, 0,  0};

const static uint8_t __linvperm[] __attribute__((aligned(64))) = {0, 8, 4, 12, 1, 9, 5, 13, 2, 10, 6, 14, 3, 11, 7, 15,
                                                                  0, 8, 4, 12, 1, 9, 5, 13, 2, 10, 6, 14, 3, 11, 7, 15};

inline void z64_to_r32_sse2(uint64_t *a, uint32_t *b) {
    /*
     * Converts a Z-ordered array of 16 uint64s, each of which encodes
     * an 8-by-8 subsquare of a 32-by-32 square, into an array of 32
     * uint32s, each of which represents a row.
     */

    asm(
        // Load bytes 0 -- 63 into registers:
        "movups (%0), %%xmm0 \n\t"
        "movups 16(%0), %%xmm3 \n\t"
        "movups 32(%0), %%xmm1 \n\t"
        "movups 48(%0), %%xmm4 \n\t"

        // Bit cycle, round I:
        "movdqa %%xmm0, %%xmm2 \n\t"
        "movdqa %%xmm3, %%xmm5 \n\t"
        "punpcklbw %%xmm1, %%xmm0 \n\t"
        "punpcklbw %%xmm4, %%xmm3 \n\t"
        "punpckhbw %%xmm1, %%xmm2 \n\t"
        "punpckhbw %%xmm4, %%xmm5 \n\t"

        // Bit cycle, round II:
        "movdqa %%xmm0, %%xmm1 \n\t"
        "movdqa %%xmm3, %%xmm4 \n\t"
        "punpcklbw %%xmm2, %%xmm0 \n\t"
        "punpcklbw %%xmm5, %%xmm3 \n\t"
        "punpckhbw %%xmm2, %%xmm1 \n\t"
        "punpckhbw %%xmm5, %%xmm4 \n\t"

        // Save bytes 0 -- 63 back into memory:
        "movups %%xmm0, 0(%1) \n\t"
        "movups %%xmm1, 16(%1) \n\t"
        "movups %%xmm3, 32(%1) \n\t"
        "movups %%xmm4, 48(%1) \n\t"

        // Load bytes 64 -- 127 into registers:
        "movups 64(%0), %%xmm0 \n\t"
        "movups 80(%0), %%xmm3 \n\t"
        "movups 96(%0), %%xmm1 \n\t"
        "movups 112(%0), %%xmm4 \n\t"

        // Bit cycle, round I:
        "movdqa %%xmm0, %%xmm2 \n\t"
        "movdqa %%xmm3, %%xmm5 \n\t"
        "punpcklbw %%xmm1, %%xmm0 \n\t"
        "punpcklbw %%xmm4, %%xmm3 \n\t"
        "punpckhbw %%xmm1, %%xmm2 \n\t"
        "punpckhbw %%xmm4, %%xmm5 \n\t"

        // Bit cycle, round II:
        "movdqa %%xmm0, %%xmm1 \n\t"
        "movdqa %%xmm3, %%xmm4 \n\t"
        "punpcklbw %%xmm2, %%xmm0 \n\t"
        "punpcklbw %%xmm5, %%xmm3 \n\t"
        "punpckhbw %%xmm2, %%xmm1 \n\t"
        "punpckhbw %%xmm5, %%xmm4 \n\t"

        // Save bytes 64 -- 127 back into memory:
        "movups %%xmm0, 64(%1) \n\t"
        "movups %%xmm1, 80(%1) \n\t"
        "movups %%xmm3, 96(%1) \n\t"
        "movups %%xmm4, 112(%1) \n\t"

        : /* no output operands -- implicitly volatile */
        : "r"(a), "r"(b)
        : "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "memory");
}

inline void z64_to_r32_centre_ssse3(uint64_t *c, uint32_t *b) {
    /*
     *    #ab#
     *    #cd# <--- [a, b, c, d]
     */

    asm(

        // Load from memory:
        "movups (%0), %%xmm0 \n\t"
        "movups 16(%0), %%xmm2 \n\t"

        // Permute bytes:
        "pshufb (%2), %%xmm0 \n\t"
        "pshufb (%2), %%xmm2 \n\t"

        // Dirty hack to perform << 8 and >> 8 during movups:
        "movups %%xmm0, 1(%1) \n\t"
        "movups %%xmm0, 15(%1) \n\t"
        "movups %%xmm2, 33(%1) \n\t"
        "movups %%xmm2, 47(%1) \n\t"

        : /* no output operands -- implicitly volatile */
        : "r"(c), "r"(b), "r"(__linvperm)
        : "xmm0", "xmm1", "xmm2", "xmm3", "memory");
}

inline void r32_centre_to_z64_ssse3(uint32_t *b, uint64_t *c) {
    /*
     * Selects the 16-by-16 square in the centre of a 32-by-32
     * square encoded as an array of rows, and converts it to a
     * Z-ordered array of 4 uint64s, each representing a 8-by-8
     * subsquare:
     *
     *    ####
     *    #ab#
     *    #cd# ---> [a, b, c, d]
     *    ####
     */

    asm(
        // Dirty hack to perform << 8 and >> 8 during movups:
        "movups 31(%1), %%xmm0 \n\t"
        "movups 49(%1), %%xmm1 \n\t"
        "movups 63(%1), %%xmm2 \n\t"
        "movups 81(%1), %%xmm3 \n\t"
        "psrld $16, %%xmm0 \n\t"
        "pslld $16, %%xmm1 \n\t"
        "psrld $16, %%xmm2 \n\t"
        "pslld $16, %%xmm3 \n\t"

        // Alternately select words from two registers:
        "por %%xmm1, %%xmm0 \n\t"
        "por %%xmm3, %%xmm2 \n\t"

        // Permute bytes:
        "pshufb (%2), %%xmm0 \n\t"
        "pshufb (%2), %%xmm2 \n\t"

        // Save back into memory:
        "movups %%xmm0, (%0) \n\t"
        "movups %%xmm2, 16(%0) \n\t"

        : /* no output operands -- implicitly volatile */
        : "r"(c), "r"(b), "r"(__lifeperm)
        : "xmm0", "xmm1", "xmm2", "xmm3", "memory");
}

inline void r32_centre_to_z64_clean(uint32_t *b, uint64_t *c) {
    /*
     * Selects the 16-by-16 square in the centre of a 32-by-32
     * square encoded as an array of rows, and converts it to a
     * Z-ordered array of 4 uint64s, each representing a 8-by-8
     * subsquare:
     *
     *    ####
     *    #ab#
     *    #cd# ---> [a, b, c, d]
     *    ####
     */

    uint32_t temp[16];

    // Загрузка данных из центра 32x32 массива
    temp[0] = (b[31] >> 16) | (b[49] << 16);
    temp[1] = (b[32] >> 16) | (b[50] << 16);
    temp[2] = (b[33] >> 16) | (b[51] << 16);
    temp[3] = (b[34] >> 16) | (b[52] << 16);
    temp[4] = (b[35] >> 16) | (b[53] << 16);
    temp[5] = (b[36] >> 16) | (b[54] << 16);
    temp[6] = (b[37] >> 16) | (b[55] << 16);
    temp[7] = (b[38] >> 16) | (b[56] << 16);
    temp[8] = (b[63] >> 16) | (b[81] << 16);
    temp[9] = (b[64] >> 16) | (b[82] << 16);
    temp[10] = (b[65] >> 16) | (b[83] << 16);
    temp[11] = (b[66] >> 16) | (b[84] << 16);
    temp[12] = (b[67] >> 16) | (b[85] << 16);
    temp[13] = (b[68] >> 16) | (b[86] << 16);
    temp[14] = (b[69] >> 16) | (b[87] << 16);
    temp[15] = (b[70] >> 16) | (b[88] << 16);

    // Применение перестановки байтов (__lifeperm)
    uint8_t *temp_bytes = reinterpret_cast<uint8_t*>(temp);
    const uint8_t *perm_mask = reinterpret_cast<const uint8_t*>(__lifeperm);
    uint8_t result[32];

    for (int i = 0; i < 16; ++i) {
        result[i] = temp_bytes[perm_mask[i]];
    }

    for (int i = 0; i < 16; ++i) {
        result[16 + i] = temp_bytes[perm_mask[16 + i]];
    }

    // Копирование данных в выходной массив
    std::memcpy(c, result, 32);
}


inline uint64_t r32_centre_to_u64(uint32_t *d, int x, int y) {
    // Not written in inline assembly for a change (!)
    uint64_t z = 0;
    for (int i = 11; i >= 4; i--) {
        z = z << 8;
        z |= (d[i + y] >> (12 + x)) & 255;
    }
    return z;
}

inline uint64_t z64_centre_to_u64(uint64_t *inleaf, int x, int y) {
    /*
     * Provided this is inlined and x, y are compile-time constants,
     * this should just involve 6 shifts, 3 ORs, and 2 ANDs:
     */
    int xs = 4 + x;
    int ys = (4 + y) << 3;
    uint64_t bitmask = (0x0101010101010101ull << xs) - 0x0101010101010101ull;
    uint64_t left = (inleaf[0] >> ys) | (inleaf[2] << (64 - ys));
    uint64_t right = (inleaf[1] >> ys) | (inleaf[3] << (64 - ys));
    uint64_t result = ((right & bitmask) << (8 - xs)) | ((left & (~bitmask)) >> xs);
    return result;
}

inline uint32_t update_row(uint32_t row_prev, uint32_t row_curr, uint32_t row_next) {
    uint32_t b = row_prev;
    uint32_t a = b << 1;
    uint32_t c = b >> 1;
    uint32_t i = row_curr;
    uint32_t h = i << 1;
    uint32_t d = i >> 1;
    uint32_t f = row_next;
    uint32_t g = f << 1;
    uint32_t e = f >> 1;

    uint32_t ab0 = a ^ b;
    uint32_t ab1 = a & b;
    uint32_t cd0 = c ^ d;
    uint32_t cd1 = c & d;

    uint32_t ef0 = e ^ f;
    uint32_t ef1 = e & f;
    uint32_t gh0 = g ^ h;
    uint32_t gh1 = g & h;

    uint32_t ad0 = ab0 ^ cd0;
    uint32_t ad1 = (ab1 ^ cd1) ^ (ab0 & cd0);
    uint32_t ad2 = ab1 & cd1;

    uint32_t eh0 = ef0 ^ gh0;
    uint32_t eh1 = (ef1 ^ gh1) ^ (ef0 & gh0);
    uint32_t eh2 = ef1 & gh1;

    uint32_t ah0 = ad0 ^ eh0;
    uint32_t xx = ad0 & eh0;
    uint32_t yy = ad1 ^ eh1;
    uint32_t ah1 = xx ^ yy;
    uint32_t ah23 = (ad2 | eh2) | (ad1 & eh1) | (xx & yy);
    uint32_t z = ~ah23 & ah1;
    uint32_t i2 = ~ah0 & z;
    uint32_t i3 = ah0 & z;
    return (i & i2) | i3;
}

inline void iterate_var_leaf32(int n, uint64_t **inleafxs, uint64_t *outleaf) {
    uint64_t inleaf[16];
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            inleaf[i * 4 + j] = inleafxs[i][j];
        }
    }

    uint32_t src[32];
    z64_to_r32_sse2(inleaf, src);

    uint32_t dst[32];

    for (int t = 1; t <= n; ++t) {
        for (int y = t; y < 32 - t; ++y) {
            dst[y] = update_row(src[y - 1], src[y], src[y + 1]);
        }
        std::swap(src, dst);
    }

    // r32_centre_to_z64_ssse3(src, outleaf);
    r32_centre_to_z64_clean(src, outleaf);
}

inline void iter4_var_leaf(uint64_t *inleaf, uint64_t *centres) {
    /*
     * Find the 8-by-8 centre after iterating a 16-by-16 leaf for a
     * further 4 iterations in the rule.
     */
    uint32_t d[16];
    uint32_t e[16];

    z64_to_r32_centre_ssse3(inleaf, d);

    // a bit overkill
    for (int t = 1; t <= 4; ++t) {
        for (int y = t; y < 16 - t; ++y) {
            e[y] = update_row(d[y - 1], d[y], d[y + 1]);
        }
        std::swap(d, e);
    }

    centres[0] = r32_centre_to_u64(d, 0, 0);
}

inline uint64_t determine_direction(uint64_t *inleaf) {
    uint64_t centre;
    iter4_var_leaf(inleaf, &centre);

    uint64_t dmap = 0;
    dmap |= ((centre == z64_centre_to_u64(inleaf, -1, -1)) ? 1 : 0);   // SE
    dmap |= ((centre == z64_centre_to_u64(inleaf, 0, -2)) ? 2 : 0);    // S
    dmap |= ((centre == z64_centre_to_u64(inleaf, 1, -1)) ? 4 : 0);    // SW
    dmap |= ((centre == z64_centre_to_u64(inleaf, 2, 0)) ? 8 : 0);     // W
    dmap |= ((centre == z64_centre_to_u64(inleaf, 1, 1)) ? 16 : 0);    // NW
    dmap |= ((centre == z64_centre_to_u64(inleaf, 0, 2)) ? 32 : 0);    // N
    dmap |= ((centre == z64_centre_to_u64(inleaf, -1, 1)) ? 64 : 0);   // NE
    dmap |= ((centre == z64_centre_to_u64(inleaf, -2, 0)) ? 128 : 0);  // E

    uint64_t lmask = 0;

    if (centre) {
        if (dmap & 170) {
            lmask |= 3;
        }
        if (dmap & 85) {
            lmask |= 7;
        }
        // if (dmap) { std::cerr << centre << " " << dmap << std::endl; }
    }

    // Use a uint64 as an ordered pair of uint32s:
    return (dmap | (lmask << 32));
}
