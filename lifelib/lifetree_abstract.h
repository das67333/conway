#pragma once

#include <fstream>

#include "hashtrees/hypertree.h"

namespace apg {

inline uint64_t transform_uint64(uint64_t x, uint8_t perm) {

    uint64_t c[4];

    c[0] = x & 0x000000000f0f0f0full;
    c[1] = (x >> 4) & 0x000000000f0f0f0full;
    c[2] = (x >> 32) & 0x000000000f0f0f0full;
    c[3] = (x >> 36) & 0x000000000f0f0f0full;

    uint64_t y = c[perm & 3] | (c[(perm >> 2) & 3] << 4) | (c[(perm >> 4) & 3] << 32) | (c[(perm >> 6) & 3] << 36);

    c[0] = y & 0x0000333300003333ull;
    c[1] = (y >> 2) & 0x0000333300003333ull;
    c[2] = (y >> 16) & 0x0000333300003333ull;
    c[3] = (y >> 18) & 0x0000333300003333ull;

    y = c[perm & 3] | (c[(perm >> 2) & 3] << 2) | (c[(perm >> 4) & 3] << 16) | (c[(perm >> 6) & 3] << 18);

    c[0] = y & 0x0055005500550055ull;
    c[1] = (y >> 1) & 0x0055005500550055ull;
    c[2] = (y >> 8) & 0x0055005500550055ull;
    c[3] = (y >> 9) & 0x0055005500550055ull;

    y = c[perm & 3] | (c[(perm >> 2) & 3] << 1) | (c[(perm >> 4) & 3] << 8) | (c[(perm >> 6) & 3] << 9);

    return y;
}

template <typename I>
struct lifemeta {
    I res;
    I aux;
};

template <typename I>
class lifetree_abstract {

public:
    uint64_t gc_threshold;

    virtual uint64_t newihandle(hypernode<I> hnode) = 0;
    virtual void delhandle(uint64_t ihandle) = 0;
    virtual void force_gc() = 0;
    virtual bool threshold_gc(uint64_t threshold) = 0;
    virtual uint64_t getcell_recurse(hypernode<I> hnode, uint64_t x, uint64_t y) = 0;
    virtual void write_macrocell(std::ostream &outstream, hypernode<I> hnode) = 0;

    bool threshold_gc() {
        return threshold_gc(gc_threshold);
    }

    virtual I make_nonleaf(uint32_t depth, nicearray<I, 4> contents) = 0;
    virtual hypernode<I> make_nonleaf_hn(uint32_t depth, nicearray<I, 4> contents) = 0;
    virtual I getpop_recurse(hypernode<I> hnode, I modprime, uint64_t layermask) = 0;
    virtual uint64_t hash(hypernode<I> hnode, bool is_root=true) = 0;

    virtual hypernode<I> getchild(hypernode<I> hnode, uint32_t n) = 0;
    virtual uint64_t leafpart(I index, uint32_t part) = 0;

    virtual hypernode<I> pyramid_down(hypernode<I> hnode) = 0;
    virtual hypernode<I> pyramid_up(hypernode<I> hnode) = 0;

    virtual hypernode<I> shift_recurse(hypernode<I> hnode, uint64_t x, uint64_t y, uint64_t exponent,
                                       std::map<std::pair<I, uint32_t>, I> *memmap) = 0;

    hypernode<I> shift_recurse(hypernode<I> hnode, uint64_t x, uint64_t y, uint64_t exponent) {
        std::map<std::pair<I, uint32_t>, I> memmap;
        return shift_recurse(hnode, x, y, exponent, &memmap);
    }

    hypernode<I> shift_toroidal(hypernode<I> hnode, int64_t x, int64_t y, uint64_t exponent) {
        nicearray<I, 4> cc = {hnode.index, hnode.index, hnode.index, hnode.index};
        hypernode<I> xcc = make_nonleaf_hn(hnode.depth + 1, cc);

        int64_t sx = x;
        int64_t sy = y;
        uint64_t sz = exponent;

        if ((sx == 0) && (sy == 0)) {
            return hnode;
        }

        while (((sx & 1) == 0) && ((sy & 1) == 0)) {
            sx = sx / 2;
            sy = sy / 2;
            sz = sz + 1;
        }

        // We cast to unsigned integers, which is okay provided our
        // universe is no larger than (2 ^ 64)-by-(2 ^ 64):
        uint64_t ux = (uint64_t)(0 - sx);
        uint64_t uy = (uint64_t)(0 - sy);

        return shift_recurse(xcc, ux, uy, sz);
    }

    hypernode<I> pyramid_up(hypernode<I> hnode_initial, uint32_t target_depth) {
        hypernode<I> hnode = hnode_initial;
        while (target_depth > hnode.depth) {
            // Do this iteratively:
            hnode = pyramid_up(hnode);
        }
        return hnode;
    }

    virtual hypernode<I> boolean_recurse(hypernode<I> lnode, hypernode<I> rnode, int operation,
                                         std::map<std::pair<std::pair<I, I>, uint32_t>, I> *memmap) = 0;

    hypernode<I> boolean_recurse(hypernode<I> lnode, hypernode<I> rnode, int operation) {
        std::map<std::pair<std::pair<I, I>, uint32_t>, I> memmap;
        return boolean_recurse(lnode, rnode, operation, &memmap);
    }

    hypernode<I> breach(hypernode<I> hnode) {
        if (hnode.index2 == 0) {
            return hnode;
        } else if (hnode.index == 0) {
            return hypernode<I>(hnode.index2, hnode.depth);
        } else {
            hypernode<I> i1(hnode.index, hnode.depth);
            hypernode<I> i2(hnode.index2, hnode.depth);
            return boolean_recurse(i1, i2, 1);
        }
    }

    hypernode<I> boolean_universe(hypernode<I> lnode, hypernode<I> rnode, int operation) {
        hypernode<I> lx = lnode;
        hypernode<I> rx = rnode;
        while (lx.depth < rx.depth) {
            lx = pyramid_up(lx);
        }
        while (lx.depth > rx.depth) {
            rx = pyramid_up(rx);
        }
        hypernode<I> hnode = boolean_recurse(lx, rx, operation);
        hnode = pyramid_down(hnode);
        return hnode;
    }

    virtual hypernode<I> iterate_recurse(hypernode<I> hnode, uint64_t mantissa, uint64_t exponent) = 0;

    hypernode<I> advance(hypernode<I> hnode_initial, uint64_t mantissa, uint64_t exponent) {
        /*
         * Advance the universe by (mantissa * (2 ** exponent)) timesteps,
         * returning an index to the resulting hypernode. This resizes the
         * universe as necessary (without changing the 'origin', taken to
         * be the centre of the hypernode).
         */
        // std::cerr << "Exponent = " << exponent << " ; mantissa = " << mantissa <<
        // std::endl;
        hypernode<I> hnode = pyramid_up(pyramid_up(hnode_initial));
        hnode = pyramid_up(hnode, exponent + 2);
        hnode = iterate_recurse(hnode, mantissa, exponent);
        hnode = pyramid_down(hnode);
        return hnode;
    }

    hypernode<I> advance(hypernode<I> hnode_initial, uint64_t steps) {
        hypernode<I> hnode = hnode_initial;
        if (steps) {
            uint64_t numsteps = steps;
            uint64_t exponent = 0;

            while ((numsteps & 7) && exponent) {
                numsteps = numsteps << 1;
                exponent -= 1;
            }

            uint64_t vm = 511;
            uint64_t mantissa = 8;
            while ((mantissa != 1) && ((numsteps % mantissa) || ((vm & (1 << mantissa)) == 0))) {
                mantissa -= 1;
            }
            if ((vm & (1 << mantissa)) == 0) {
                std::cerr << "Rule b3s23 cannot be iterated " << steps;
                std::cerr << " x 2^stepexp generations." << std::endl;
                exit(1);
            }

            uint64_t multiplier = numsteps / mantissa;

            while (multiplier) {
                if (multiplier & 1) {
                    hnode = advance(hnode, mantissa, exponent);
                }
                multiplier = multiplier >> 1;
                exponent += 1;
            }
        } else {
            hnode = pyramid_down(hnode);
        }
        return hnode;
    }

    hypernode<I> shift_universe(hypernode<I> hnode_initial, int64_t x, int64_t y, uint64_t exponent) {
        hypernode<I> hnode = hnode_initial;
        if ((x != 0) || (y != 0)) {
            int64_t absx = (x < 0) ? (-x) : x;
            int64_t absy = (y < 0) ? (-y) : y;
            uint64_t diameter = (absx > absy) ? absx : absy;
            uint64_t lzcount = __builtin_clzll(diameter);
            hnode = pyramid_up(hnode_initial, (64 - lzcount) + exponent);
            hnode = pyramid_up(hnode);
            hnode = shift_toroidal(hnode, x, y, exponent);
        }
        hnode = pyramid_down(hnode);
        return hnode;
    }

    hypernode<I> shift_universe(hypernode<I> hnode_initial, int64_t x, int64_t y) {
        return shift_universe(hnode_initial, x, y, 0);
    }

    virtual hypernode<I> read_macrocell(std::istream &instream, std::map<uint64_t, uint64_t> *lmap) = 0;

    hypernode<I> load_macrocell(std::string filename) {
        std::ifstream f(filename);
        std::map<uint64_t, uint64_t> lmap;
        lmap[0] = 0;
        lmap[1] = 3;
        lmap[2] = 2;
        lmap[3] = 11;
        lmap[4] = 10;
        lmap[5] = 15;
        return read_macrocell(f, &lmap);
    }

    hypernode<I> load_macrocell(std::string filename, int64_t x, int64_t y) {
        return shift_universe(load_macrocell(filename), x, y);
    }
};
}  // namespace apg
