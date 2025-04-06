#pragma once

#include <sstream>
#include <unordered_map>

#include "leaf_iterators.h"
#include "lifetree_abstract.h"

template <typename I>
struct Key {
    I index;
    uint32_t depth;

    bool operator==(const Key<I> &) const = default;
};

namespace std {

template <typename I>
struct hash<Key<I>> {
    size_t operator()(const Key<I> &x) const {
        return size_t{x.index} | (size_t{x.depth} >> 32);
    }
};

}  // namespace std

namespace apg {

template <typename I, typename J>
class lifetree_generic : public lifetree_abstract<I> {

public:
    hypertree<I, 4, J, nicearray<uint64_t, 4>, J> htree;

    std::unordered_map<Key<I>, uint64_t> hash_cache;

    using lifetree_abstract<I>::breach;

    uint64_t newihandle(hypernode<I> hnode) override {
        uint64_t x = ++htree.hcounter;
        htree.ihandles.emplace(x, hnode);
        return x;
    }
    void delhandle(uint64_t ihandle) override {
        htree.ihandles.erase(ihandle);
    }

    uint64_t total_bytes() {
        return htree.total_bytes();
    }
    void force_gc() override {
        htree.gc_full();
    }
    virtual bool threshold_gc(uint64_t threshold) override = 0;

    kiventry<nicearray<I, 4>, I, J> *ind2ptr_nonleaf(uint32_t depth, I index) {
        return htree.ind2ptr_nonleaf(depth, index);
    }
    kiventry<nicearray<uint64_t, 4>, I, J> *ind2ptr_leaf(I index) {
        return htree.ind2ptr_leaf(index);
    }
    I make_nonleaf(uint32_t depth, nicearray<I, 4> contents) override {
        return htree.make_nonleaf(depth, contents);
    }
    hypernode<I> make_nonleaf_hn(uint32_t depth, nicearray<I, 4> contents) override {
        return htree.make_nonleaf_hn(depth, contents);
    }
    I make_leaf(nicearray<uint64_t, 4> contents) {
        return htree.make_leaf(contents);
    }
    hypernode<I> getchild(hypernode<I> parent, uint32_t n) override {
        return htree.getchild(parent, n);
    }

    uint64_t leafpart(I index, uint32_t part) override {
        kiventry<nicearray<uint64_t, 4>, I, J> *pptr = ind2ptr_leaf(index);
        if (part < 4) {
            return pptr->key[part];
        } else {
            return 0;
        }
    }

    virtual hypernode<I> iterate_recurse(hypernode<I> hnode, uint64_t mantissa, uint64_t exponent) override = 0;

    uint64_t write_macrocell_leaf(std::ostream &outstream, uint64_t leaf, std::map<uint64_t, uint64_t> *subleaf2int,
                                  uint64_t &linenum) {

        auto it = subleaf2int->find(leaf);
        if (leaf == 0) {
            return 0;
        } else if (it != subleaf2int->end()) {
            return it->second;
        } else {
            uint64_t x = leaf;
            for (int i = 0; i < 8; i++) {
                for (int j = 0; j < 8; j++) {
                    outstream << ".*"[x & 1];
                    x = x >> 1;
                }
                outstream << "$";
            }
            outstream << std::endl;
            subleaf2int->emplace(leaf, (++linenum));
            return linenum;
        }
    }

    uint64_t write_macrocell_recurse(std::ostream &outstream, hypernode<I> hnode,
                                     std::map<uint64_t, uint64_t> *subleaf2int,
                                     std::map<std::pair<I, uint32_t>, uint64_t> *hnode2int, uint64_t &linenum) {
        /*
         * Writes a 2-state macrocell file according to the contents of
         * layer 0.
         */
        auto it = hnode2int->find(std::make_pair(hnode.index, hnode.depth));

        if (hnode.index == 0) {
            return 0;
        } else if (it != hnode2int->end()) {
            return it->second;
        } else if (hnode.depth == 0) {
            // Extract the pointer to the node:
            kiventry<nicearray<uint64_t, 4>, I, J> *pptr = ind2ptr_leaf(hnode.index);
            uint64_t a = write_macrocell_leaf(outstream, pptr->key[0], subleaf2int, linenum);
            uint64_t b = write_macrocell_leaf(outstream, pptr->key[1], subleaf2int, linenum);
            uint64_t c = write_macrocell_leaf(outstream, pptr->key[2], subleaf2int, linenum);
            uint64_t d = write_macrocell_leaf(outstream, pptr->key[3], subleaf2int, linenum);
            outstream << (hnode.depth + 4) << " " << a << " " << b << " " << c << " " << d << std::endl;
            hnode2int->emplace(std::make_pair(hnode.index, hnode.depth), (++linenum));
            return linenum;
        } else {
            uint64_t a = write_macrocell_recurse(outstream, getchild(hnode, 0), subleaf2int, hnode2int, linenum);
            uint64_t b = write_macrocell_recurse(outstream, getchild(hnode, 1), subleaf2int, hnode2int, linenum);
            uint64_t c = write_macrocell_recurse(outstream, getchild(hnode, 2), subleaf2int, hnode2int, linenum);
            uint64_t d = write_macrocell_recurse(outstream, getchild(hnode, 3), subleaf2int, hnode2int, linenum);
            outstream << (hnode.depth + 4) << " " << a << " " << b << " " << c << " " << d << std::endl;
            hnode2int->emplace(std::make_pair(hnode.index, hnode.depth), (++linenum));
            return linenum;
        }
    }

    void write_macrocell(std::ostream &outstream, hypernode<I> hnode) override {
        outstream << "[M2] (lifelib ll1.65)" << std::endl;
        std::map<uint64_t, uint64_t> subleaf2int;
        std::map<std::pair<I, uint32_t>, uint64_t> hnode2int;
        uint64_t linenum = 0;
        write_macrocell_recurse(outstream, breach(hnode), &subleaf2int, &hnode2int, linenum);
    }

    hypernode<I> read_macrocell(std::istream &instream, std::map<uint64_t, uint64_t> *lmap) override {
        /*
         * Returns a hypernode representing the contents of a macrocell
         * file. This handles both 2- and n-state macrocell files using the
         * same code.
         *
         * lmap should be a pointer to a map which translates the states
         * given in the macrocell file to a bit-array in memory. For
         * instance, if state 7 should map to 'on' in layers 0, 3 and 4,
         * then the map should have:
         *
         * (*lmap)[7] = 0b00011001 = 25
         *
         * Zero must always map to zero. If lmap is a null pointer, then it
         * is taken to be the identity map.
         */

        std::string line;                             // line read from file
        std::vector<nicearray<uint64_t, 1>> pleaves;  // partial leaves
        nicearray<uint64_t, 1> zeropleaf = {0ull};    // empty partial leaf
        pleaves.push_back(zeropleaf);                 // zero means zero

        uint32_t log2size = 0;  // log2(size) of most recent node
        I lastnode = -1;        // index of most recent node

        std::ostringstream rlestream;

        while (std::getline(instream, line)) {
            if (line.empty() || line[0] == '#' || line[0] == '[') {
                continue;
            } else {
                nicearray<uint64_t, 1> pleaf = {0ull};
                if (line[0] == '.' || line[0] == '*' || line[0] == '$') {
                    uint64_t lm = (lmap == 0) ? 1 : ((*lmap)[1]);
                    uint64_t pl = 0;
                    uint64_t x = 0;
                    uint64_t y = 0;
                    // Load the 8-by-8 pixel representation into uint64_t pl:
                    for (unsigned int i = 0; i < line.length(); i++) {
                        if (line[i] == '$') {
                            x = 0;
                            y += 1;
                        } else {
                            if (line[i] == '*') {
                                pl |= (1ull << (x + 8 * y));
                            }
                            x += 1;
                        }
                    }
                    if (pl == 0) {
                        std::cerr << "Warning: " << line << std::endl;
                    }
                    // Populate the partial leaf according to the leafmap:
                    for (unsigned int i = 0; i < 1; i++) {
                        if (lm & 1) {
                            pleaf[i] = pl;
                        } else {
                            pleaf[i] = 0;
                        }
                        lm = lm >> 1;
                    }
                } else if (line[0] >= '1' && line[0] <= '9') {

                    // Line should be a space-separated list of 5 integers:
                    std::stringstream s(line);
                    uint64_t a = 0, b = 0, c = 0, d = 0;
                    s >> log2size >> a >> b >> c >> d;

                    if (log2size == 1) {
                        uint64_t ilma = (lmap == 0) ? a : ((*lmap)[a]);
                        uint64_t ilmb = (lmap == 0) ? b : ((*lmap)[b]);
                        uint64_t ilmc = (lmap == 0) ? c : ((*lmap)[c]);
                        uint64_t ilmd = (lmap == 0) ? d : ((*lmap)[d]);
                        for (unsigned int i = 0; i < 1; i++) {
                            pleaf[i] = (ilma & 1) | ((ilmb & 1) << 1) | ((ilmc & 1) << 8) | ((ilmd & 1) << 9);
                            ilma = ilma >> 1;
                            ilmb = ilmb >> 1;
                            ilmc = ilmc >> 1;
                            ilmd = ilmd >> 1;
                        }
                    } else if (log2size == 2) {
                        for (unsigned int i = 0; i < 1; i++) {
                            pleaf[i] = pleaves[a][i];
                            pleaf[i] |= (pleaves[b][i] << 2);
                            pleaf[i] |= (pleaves[c][i] << 16);
                            pleaf[i] |= (pleaves[d][i] << 18);
                        }
                    } else if (log2size == 3) {
                        for (unsigned int i = 0; i < 1; i++) {
                            pleaf[i] = pleaves[a][i];
                            pleaf[i] |= (pleaves[b][i] << 4);
                            pleaf[i] |= (pleaves[c][i] << 32);
                            pleaf[i] |= (pleaves[d][i] << 36);
                        }
                    } else if (log2size == 4) {
                        // Leaf:
                        nicearray<uint64_t, 4 * 1> leaf;
                        for (unsigned int i = 0; i < 1; i++) {
                            leaf[4 * i] = pleaves[a][i];
                            leaf[4 * i + 1] = pleaves[b][i];
                            leaf[4 * i + 2] = pleaves[c][i];
                            leaf[4 * i + 3] = pleaves[d][i];
                        }
                        lastnode = make_leaf(leaf);
                        pleaf[0] = lastnode;
                    } else {
                        // Nonleaf:
                        I tl = pleaves[a][0];
                        I tr = pleaves[b][0];
                        I bl = pleaves[c][0];
                        I br = pleaves[d][0];
                        nicearray<I, 4> nonleaf = {tl, tr, bl, br};
                        lastnode = make_nonleaf(log2size - 4, nonleaf);
                        pleaf[0] = lastnode;
                    }
                } else {
                    std::cerr << "Invalid line: " << line << std::endl;
                    continue;
                }
                pleaves.push_back(pleaf);
            }
        }

        return hypernode<I>(lastnode, log2size - 4);
    }

    // Extract the (x, y)th cell from a node:
    uint64_t getcell_recurse(hypernode<I> hnode, uint64_t x, uint64_t y) override {
        if (hnode.index == 0) {
            return 0;
        } else if (hnode.depth == 0) {
            kiventry<nicearray<uint64_t, 4>, I, J> *pptr = ind2ptr_leaf(hnode.index);
            uint64_t c = 0;
            for (unsigned int i = 0; i < 1; i++) {
                uint64_t rel = pptr->key[4 * i + ((y & 8) >> 2) + ((x & 8) >> 3)];
                rel = rel >> (x & 7);
                rel = rel >> ((y & 7) << 3);
                c |= ((rel & 1) << i);
            }
            return c;
        } else {
            uint64_t tx = (x >> (hnode.depth + 3)) & 1;
            uint64_t ty = (y >> (hnode.depth + 3)) & 1;
            return getcell_recurse(getchild(hnode, tx + 2 * ty), x, y);
        }
    }

    I getpop_recurse(hypernode<I> hnode, I modprime, uint64_t layermask) override {
        /*
         * Compute the population mod p of a given hypernode. A cell of
         * state S is considered alive if and only if (layermask & S) != 0
         * where & indicates bitwise conjunction. Equivalently, we take the
         * population of the union of the layers indexed by the bits in
         * layermask.
         *
         * If the layermask is changed, you should run a garbage-collection
         * so as to clear the memoized population counts.
         */

        if (hnode.index2 != 0) {

            return getpop_recurse(breach(hnode), modprime, layermask);

        } else if (hnode.index == 0) {

            // Empty nodes have population 0:
            return 0;

        } else if (hnode.depth == 0) {

            // This is a leaf node (16-by-16 square); extract its memory location:
            kiventry<nicearray<uint64_t, 4>, I, J> *pptr = ind2ptr_leaf(hnode.index);

            if (pptr->gcflags & 1) {
                // We've cached the population; return it:
                return pptr->value.aux;
            } else {
                // Accumulate the popcount of the four 8-by-8 subleaves:
                I pop = 0;
                uint64_t a = 0, b = 0, c = 0, d = 0;
                for (unsigned int i = 0; i < 1; i++) {
                    if (layermask & (1ull << i)) {
                        a |= pptr->key[4 * i];
                        b |= pptr->key[4 * i + 1];
                        c |= pptr->key[4 * i + 2];
                        d |= pptr->key[4 * i + 3];
                    }
                }
                pop += __builtin_popcountll(a);
                pop += __builtin_popcountll(b);
                pop += __builtin_popcountll(c);
                pop += __builtin_popcountll(d);
                pptr->value.aux = pop;
                pptr->gcflags |= 1;
                return pop;
            }

        } else {

            // Non-leaf node (32-by-32 or larger):
            kiventry<nicearray<I, 4>, I, J> *pptr = ind2ptr_nonleaf(hnode.depth, hnode.index);
            I oldflags = pptr->gcflags;

            // Determine whether our cached value for the population is correct.
            // If the depth is <= 11 (i.e. 32768-by-32768 or smaller), then the
            // population is smaller than any of the prime moduli we ever use
            // (namely 2^30 + k):
            bool goodpop = (hnode.depth <= 11) ? (oldflags & 1) : (((oldflags ^ modprime) & 0x1ff) == 0);

            if (goodpop) {
                // Return cached result:
                return pptr->value.aux;
            } else {
                // Recompute population recursively:
                I pop = 0;
                for (int i = 0; i < 4; i++) {
                    pop += getpop_recurse(hypernode<I>(pptr->key[i], hnode.depth - 1), modprime, layermask);
                    pop %= modprime;
                }
                pptr->value.aux = pop;
                oldflags ^= (oldflags & 0x1ff);
                oldflags |= (modprime & 0x1ff);
                pptr->gcflags = oldflags;
                return pop;
            }
        }
    }

    uint64_t hash(hypernode<I> hnode, bool is_root) override {
        if (is_root) {
            hnode = this->breach(hnode);
        }

        if (auto it = hash_cache.find({hnode.index, hnode.depth}); it != hash_cache.end()) {
            return it->second;
        }

        auto combine = [](uint64_t x, uint64_t y) -> uint64_t { return x ^ (y + 0x9e3779b9 + (x << 6) + (x >> 2)); };

        uint64_t result = 0;
        if (hnode.depth == 0) {
            kiventry<nicearray<uint64_t, 4>, I, J> *pptr = ind2ptr_leaf(hnode.index);
            for (int i = 0; i != 4; ++i) {
                result = combine(result, pptr->key[i]);
            }
        } else {
            kiventry<nicearray<I, 4>, I, J> *pptr = ind2ptr_nonleaf(hnode.depth, hnode.index);
            for (int i = 0; i != 4; ++i) {
                result = combine(result, hash(hypernode(pptr->key[i], hnode.depth - 1), false));
            }
        }
        hash_cache.insert_or_assign({hnode.index, hnode.depth}, result);

        if (is_root) {
            hash_cache.clear();
        }
        return result;
    }

    hypernode<I> subnode(hypernode<I> hnode, uint64_t x, uint64_t y, uint64_t n) {
        hypernode<I> hnode2 = hnode;
        uint64_t i = n;
        while (i-- > 0) {
            uint64_t tx = (x >> i) & 1;
            uint64_t ty = (y >> i) & 1;
            hnode2 = getchild(hnode2, tx + 2 * ty);
        }
        return hnode2;
    }

    hypernode<I> shift_recurse(hypernode<I> hnode, uint64_t x, uint64_t y, uint64_t exponent,
                               std::map<std::pair<I, uint32_t>, I> *memmap) override {

        if (hnode.index2 != 0) {
            return shift_recurse(breach(hnode), x, y, exponent, memmap);
        }

        auto it = memmap->find(std::make_pair(hnode.index, hnode.depth));

        if (hnode.index == 0) {
            return hypernode<I>(0, hnode.depth - 1);
        } else if (it != memmap->end()) {
            return hypernode<I>(it->second, hnode.depth - 1);
        } else {

            // Extract the pointer to the node:
            kiventry<nicearray<I, 4>, I, J> *pptr = ind2ptr_nonleaf(hnode.depth, hnode.index);

            if (hnode.depth + 2 < exponent) {

                // Shift by zero:
                I res = pptr->key[0];
                memmap->emplace(std::make_pair(hnode.index, hnode.depth), res);
                return hypernode<I>(res, hnode.depth - 1);

            } else if (hnode.depth > 1) {

                // We want to do sign-extended right-shift:
                uint64_t bs = hnode.depth + 2 - exponent;
                bs = (bs < 64) ? bs : 63;
                uint64_t tx = (x >> bs) & 1;
                uint64_t ty = (y >> bs) & 1;

                // Extract the pointers for the children:
                kiventry<nicearray<I, 4>, I, J> *pptr_tl = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[0]);
                kiventry<nicearray<I, 4>, I, J> *pptr_tr = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[1]);
                kiventry<nicearray<I, 4>, I, J> *pptr_bl = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[2]);
                kiventry<nicearray<I, 4>, I, J> *pptr_br = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[3]);

                hypernode<I> xtl, xtr, xbl, xbr;

                if (ty) {
                    if (tx) {
                        nicearray<I, 4> tl2 = {pptr_tl->key[3], pptr_tr->key[2], pptr_bl->key[1], pptr_br->key[0]};
                        nicearray<I, 4> tr2 = {pptr_tr->key[2], pptr_tr->key[3], pptr_br->key[0], pptr_br->key[1]};
                        nicearray<I, 4> bl2 = {pptr_bl->key[1], pptr_br->key[0], pptr_bl->key[3], pptr_br->key[2]};
                        nicearray<I, 4> br2 = {pptr_br->key[0], pptr_br->key[1], pptr_br->key[2], pptr_br->key[3]};
                        xtl = make_nonleaf_hn(hnode.depth - 1, tl2);
                        xtr = make_nonleaf_hn(hnode.depth - 1, tr2);
                        xbl = make_nonleaf_hn(hnode.depth - 1, bl2);
                        xbr = make_nonleaf_hn(hnode.depth - 1, br2);
                    } else {
                        nicearray<I, 4> tl2 = {pptr_tl->key[2], pptr_tl->key[3], pptr_bl->key[0], pptr_bl->key[1]};
                        nicearray<I, 4> tr2 = {pptr_tl->key[3], pptr_tr->key[2], pptr_bl->key[1], pptr_br->key[0]};
                        nicearray<I, 4> bl2 = {pptr_bl->key[0], pptr_bl->key[1], pptr_bl->key[2], pptr_bl->key[3]};
                        nicearray<I, 4> br2 = {pptr_bl->key[1], pptr_br->key[0], pptr_bl->key[3], pptr_br->key[2]};
                        xtl = make_nonleaf_hn(hnode.depth - 1, tl2);
                        xtr = make_nonleaf_hn(hnode.depth - 1, tr2);
                        xbl = make_nonleaf_hn(hnode.depth - 1, bl2);
                        xbr = make_nonleaf_hn(hnode.depth - 1, br2);
                    }
                } else {
                    if (tx) {
                        nicearray<I, 4> tl2 = {pptr_tl->key[1], pptr_tr->key[0], pptr_tl->key[3], pptr_tr->key[2]};
                        nicearray<I, 4> tr2 = {pptr_tr->key[0], pptr_tr->key[1], pptr_tr->key[2], pptr_tr->key[3]};
                        nicearray<I, 4> bl2 = {pptr_tl->key[3], pptr_tr->key[2], pptr_bl->key[1], pptr_br->key[0]};
                        nicearray<I, 4> br2 = {pptr_tr->key[2], pptr_tr->key[3], pptr_br->key[0], pptr_br->key[1]};
                        xtl = make_nonleaf_hn(hnode.depth - 1, tl2);
                        xtr = make_nonleaf_hn(hnode.depth - 1, tr2);
                        xbl = make_nonleaf_hn(hnode.depth - 1, bl2);
                        xbr = make_nonleaf_hn(hnode.depth - 1, br2);
                    } else {
                        nicearray<I, 4> tl2 = {pptr_tl->key[0], pptr_tl->key[1], pptr_tl->key[2], pptr_tl->key[3]};
                        nicearray<I, 4> tr2 = {pptr_tl->key[1], pptr_tr->key[0], pptr_tl->key[3], pptr_tr->key[2]};
                        nicearray<I, 4> bl2 = {pptr_tl->key[2], pptr_tl->key[3], pptr_bl->key[0], pptr_bl->key[1]};
                        nicearray<I, 4> br2 = {pptr_tl->key[3], pptr_tr->key[2], pptr_bl->key[1], pptr_br->key[0]};
                        xtl = make_nonleaf_hn(hnode.depth - 1, tl2);
                        xtr = make_nonleaf_hn(hnode.depth - 1, tr2);
                        xbl = make_nonleaf_hn(hnode.depth - 1, bl2);
                        xbr = make_nonleaf_hn(hnode.depth - 1, br2);
                    }
                }

                hypernode<I> ytl = shift_recurse(xtl, x, y, exponent, memmap);
                hypernode<I> ytr = shift_recurse(xtr, x, y, exponent, memmap);
                hypernode<I> ybl = shift_recurse(xbl, x, y, exponent, memmap);
                hypernode<I> ybr = shift_recurse(xbr, x, y, exponent, memmap);
                nicearray<I, 4> cc2 = {ytl.index, ytr.index, ybl.index, ybr.index};
                hypernode<I> xcc = make_nonleaf_hn(hnode.depth - 1, cc2);
                memmap->emplace(std::make_pair(hnode.index, hnode.depth), xcc.index);
                return xcc;

            } else {

                // We have a 32-by-32 square:
                uint64_t tx = (exponent < 4) ? ((x << exponent) & 15) : 0;
                uint64_t ty = (exponent < 4) ? ((y << exponent) & 15) : 0;
                nicearray<uint64_t, 4> outleaf = {0ull};

                for (int j = 0; j < 1; j++) {
                    uint64_t inleaves[16];
                    for (int i = 0; i < 4; i++) {
                        std::memcpy(inleaves + (4 * i), &ind2ptr_leaf(pptr->key[i])->key[4 * j], 32);
                    }
                    uint32_t d[32];
                    z64_to_r32_sse2(inleaves, d);
                    uint32_t e[32];
                    for (int i = 0; i < 16; i++) {
                        e[i + 8] = (d[i + ty] >> tx) << 8;
                    }
                    r32_centre_to_z64_ssse3(e, &outleaf[4 * j]);
                }

                I res = make_leaf(outleaf);
                memmap->emplace(std::make_pair(hnode.index, hnode.depth), res);
                return hypernode<I>(res, 0);
            }
        }
    }

    hypernode<I> boolean_recurse(hypernode<I> lnode, hypernode<I> rnode, int operation,
                                 std::map<std::pair<std::pair<I, I>, uint32_t>, I> *memmap) override {
        /*
         *   0 = and
         *   1 = or
         *   2 = xor
         *   3 = andn
         */

        if ((lnode.index == 0) && (lnode.index2 == 0)) {
            if (operation == 0 || operation == 3) {
                return lnode;
            } else {
                return rnode;
            }
        } else if ((rnode.index == 0) && (rnode.index2 == 0)) {
            if (operation == 0) {
                return rnode;
            } else {
                return lnode;
            }
        } else if ((rnode.index2 != 0) || (lnode.index2 != 0)) {
            return boolean_recurse(breach(lnode), breach(rnode), operation, memmap);
        } else {
            // Both operands are nonzero, so we need to actually compute
            // the result recursively. Firstly, we check to see whether
            // the result has already been computed and cached:
            auto it = memmap->find(std::make_pair(std::make_pair(lnode.index, rnode.index), lnode.depth));
            if (it != memmap->end()) {
                return hypernode<I>(it->second, lnode.depth);
            } else if (lnode.depth >= 1) {
                // Nonleaf node:
                kiventry<nicearray<I, 4>, I, J> *lptr = ind2ptr_nonleaf(lnode.depth, lnode.index);
                kiventry<nicearray<I, 4>, I, J> *rptr = ind2ptr_nonleaf(rnode.depth, rnode.index);
                hypernode<I> ytl = boolean_recurse(hypernode<I>(lptr->key[0], lnode.depth - 1),
                                                   hypernode<I>(rptr->key[0], rnode.depth - 1), operation, memmap);
                hypernode<I> ytr = boolean_recurse(hypernode<I>(lptr->key[1], lnode.depth - 1),
                                                   hypernode<I>(rptr->key[1], rnode.depth - 1), operation, memmap);
                hypernode<I> ybl = boolean_recurse(hypernode<I>(lptr->key[2], lnode.depth - 1),
                                                   hypernode<I>(rptr->key[2], rnode.depth - 1), operation, memmap);
                hypernode<I> ybr = boolean_recurse(hypernode<I>(lptr->key[3], lnode.depth - 1),
                                                   hypernode<I>(rptr->key[3], rnode.depth - 1), operation, memmap);
                nicearray<I, 4> cc = {ytl.index, ytr.index, ybl.index, ybr.index};
                hypernode<I> xcc = make_nonleaf_hn(lnode.depth, cc);
                memmap->emplace(std::make_pair(std::make_pair(lnode.index, rnode.index), lnode.depth), xcc.index);
                return xcc;
            } else {
                // Leaf node:
                kiventry<nicearray<uint64_t, 4>, I, J> *lptr = ind2ptr_leaf(lnode.index);
                kiventry<nicearray<uint64_t, 4>, I, J> *rptr = ind2ptr_leaf(rnode.index);
                nicearray<uint64_t, 4> outleaf = {0ull};
                if (operation == 0) {
                    for (int i = 0; i < 4; i++) {
                        outleaf[i] = lptr->key[i] & rptr->key[i];
                    }
                } else if (operation == 1) {
                    for (int i = 0; i < 4; i++) {
                        outleaf[i] = lptr->key[i] | rptr->key[i];
                    }
                } else if (operation == 2) {
                    for (int i = 0; i < 4; i++) {
                        outleaf[i] = lptr->key[i] ^ rptr->key[i];
                    }
                } else {
                    for (int i = 0; i < 4; i++) {
                        outleaf[i] = lptr->key[i] & ~(rptr->key[i]);
                    }
                }
                I res = make_leaf(outleaf);
                memmap->emplace(std::make_pair(std::make_pair(lnode.index, rnode.index), 0), res);
                return hypernode<I>(res, 0);
            }
        }
    }

    hypernode<I> pyramid_up(hypernode<I> hnode) override {

        if (hnode.index2 != 0) {
            hypernode<I> i1 = pyramid_up(hypernode<I>(hnode.index, hnode.depth));
            hypernode<I> i2 = pyramid_up(hypernode<I>(hnode.index2, hnode.depth));
            return hypernode<I>(i1.index, i2.index, i1.depth);
        }

        I z = 0;

        if (hnode.depth == 0) {
            nicearray<I, 4> cc = {z, z, z, hnode.index};
            hypernode<I> hnode2 = make_nonleaf_hn(hnode.depth + 1, cc);
            return this->shift_toroidal(hnode2, -1, -1, 3);
        } else {
            kiventry<nicearray<I, 4>, I, J> *pptr = ind2ptr_nonleaf(hnode.depth, hnode.index);
            nicearray<I, 4> tl = {z, z, z, pptr->key[0]};
            nicearray<I, 4> tr = {z, z, pptr->key[1], z};
            nicearray<I, 4> bl = {z, pptr->key[2], z, z};
            nicearray<I, 4> br = {pptr->key[3], z, z, z};
            nicearray<I, 4> nc = {make_nonleaf(hnode.depth, tl), make_nonleaf(hnode.depth, tr),
                                  make_nonleaf(hnode.depth, bl), make_nonleaf(hnode.depth, br)};
            return make_nonleaf_hn(hnode.depth + 1, nc);
        }
    }

    hypernode<I> pyramid_down(hypernode<I> hnode) override {

        if (hnode.depth <= 1) {
            return hnode;
        }

        if (hnode.index2 != 0) {
            hypernode<I> i1 = pyramid_down(hypernode<I>(hnode.index, hnode.depth));
            hypernode<I> i2 = pyramid_down(hypernode<I>(hnode.index2, hnode.depth));
            while (i1.depth < i2.depth) {
                i1 = pyramid_up(i1);
            }
            while (i2.depth < i1.depth) {
                i2 = pyramid_up(i2);
            }
            return hypernode<I>(i1.index, i2.index, i1.depth);
        }

        if (hnode.index == 0) {
            return hypernode<I>(0, 1);
        }

        // Extract the pointer for the node and its children:
        kiventry<nicearray<I, 4>, I, J> *pptr = ind2ptr_nonleaf(hnode.depth, hnode.index);
        kiventry<nicearray<I, 4>, I, J> *pptr_tl = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[0]);
        kiventry<nicearray<I, 4>, I, J> *pptr_tr = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[1]);
        kiventry<nicearray<I, 4>, I, J> *pptr_bl = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[2]);
        kiventry<nicearray<I, 4>, I, J> *pptr_br = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[3]);

        bool tl_good = (pptr_tl->key[0] == 0) && (pptr_tl->key[1] == 0) && (pptr_tl->key[2] == 0);
        bool tr_good = (pptr_tr->key[0] == 0) && (pptr_tr->key[1] == 0) && (pptr_tr->key[3] == 0);
        bool bl_good = (pptr_bl->key[0] == 0) && (pptr_bl->key[2] == 0) && (pptr_bl->key[3] == 0);
        bool br_good = (pptr_br->key[1] == 0) && (pptr_br->key[2] == 0) && (pptr_br->key[3] == 0);

        if (tl_good && tr_good && bl_good && br_good) {
            nicearray<I, 4> cc = {pptr_tl->key[3], pptr_tr->key[2], pptr_bl->key[1], pptr_br->key[0]};
            hypernode<I> hncc = make_nonleaf_hn(hnode.depth - 1, cc);
            // Do this recursively:
            return pyramid_down(hncc);
        } else {
            return hnode;
        }
    }

    nicearray<I, 9> ninechildren(hypernode<I> hnode) {

        // Extract the pointer to the node:
        auto pptr = ind2ptr_nonleaf(hnode.depth, hnode.index);

        // Extract the pointers for the children:
        kiventry<nicearray<I, 4>, I, J> *pptr_tl = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[0]);
        kiventry<nicearray<I, 4>, I, J> *pptr_tr = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[1]);
        kiventry<nicearray<I, 4>, I, J> *pptr_bl = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[2]);
        kiventry<nicearray<I, 4>, I, J> *pptr_br = ind2ptr_nonleaf(hnode.depth - 1, pptr->key[3]);
        nicearray<I, 4> cc = {pptr_tl->key[3], pptr_tr->key[2], pptr_bl->key[1], pptr_br->key[0]};
        nicearray<I, 4> tc = {pptr_tl->key[1], pptr_tr->key[0], pptr_tl->key[3], pptr_tr->key[2]};
        nicearray<I, 4> bc = {pptr_bl->key[1], pptr_br->key[0], pptr_bl->key[3], pptr_br->key[2]};
        nicearray<I, 4> cl = {pptr_tl->key[2], pptr_tl->key[3], pptr_bl->key[0], pptr_bl->key[1]};
        nicearray<I, 4> cr = {pptr_tr->key[2], pptr_tr->key[3], pptr_br->key[0], pptr_br->key[1]};

        nicearray<I, 9> res = {pptr->key[0],
                               make_nonleaf(hnode.depth - 1, tc),
                               pptr->key[1],
                               make_nonleaf(hnode.depth - 1, cl),
                               make_nonleaf(hnode.depth - 1, cc),
                               make_nonleaf(hnode.depth - 1, cr),
                               pptr->key[2],
                               make_nonleaf(hnode.depth - 1, bc),
                               pptr->key[3]};

        return res;
    }

    nicearray<I, 4> fourchildren(hypernode<I> hnode, nicearray<I, 9> frags) {

        auto fragments = &frags[0];

        nicearray<I, 4> tl = {fragments[0], fragments[1], fragments[3], fragments[4]};
        nicearray<I, 4> tr = {fragments[1], fragments[2], fragments[4], fragments[5]};
        nicearray<I, 4> bl = {fragments[3], fragments[4], fragments[6], fragments[7]};
        nicearray<I, 4> br = {fragments[4], fragments[5], fragments[7], fragments[8]};

        nicearray<I, 4> res = {make_nonleaf(hnode.depth - 1, tl), make_nonleaf(hnode.depth - 1, tr),
                               make_nonleaf(hnode.depth - 1, bl), make_nonleaf(hnode.depth - 1, br)};

        return res;
    }

    hypernode<I> iterate_recurse1(hypernode<I> hnode, uint64_t mantissa, uint64_t exponent) {
        /*
         * Given a 2^n-by-2^n square represented by a hypernode, return the
         * central 2^(n-1)-by-2^(n-1) subsquare advanced by M * (2 ** E)
         * generations.
         *
         * This uses Gosper's HashLife algorithm down to a base-case where
         * n = 5 (i.e. computing the 16-by-16 interior of a 32-by-32 grid)
         * is performed by vectorised bitsliced assembly code.
         */
        if (hnode.index == 0) {

            // Node is empty; return an empty node of the next size down:
            return hypernode<I>(0, hnode.depth - 1);
        }

        // Extract the pointer to the node:
        kiventry<nicearray<I, 4>, I, J> *pptr = ind2ptr_nonleaf(hnode.depth, hnode.index);

        // Determine whether 1 or 2 stages are necessary:
        bool bothstages = (hnode.depth <= (1 + exponent));

        // Return the result if we've previously cached it:
        uint64_t gcdesc = pptr->gcflags >> 9;
        if ((gcdesc & 7) == (mantissa - 1) && (0 == ((gcdesc >> 3) & 15))) {
            uint64_t gcexp = gcdesc >> 7;
            if (gcexp == (1 + exponent) || (bothstages && (gcexp >= hnode.depth))) {
                // The exponent and mantissa are compatible with their desired values:
                return hypernode<I>(pptr->value.res, hnode.depth - 1);
            }
        }

        if (hnode.depth == 1) {

            // Set up the memory locations:
            nicearray<uint64_t, 4> outleaf = {0ull};

            uint64_t *inleafxs[4];

            for (int i = 0; i < 4; i++) {
                inleafxs[i] = &ind2ptr_leaf(pptr->key[i])->key[0];
            }

            iterate_var_leaf32(mantissa, inleafxs, &outleaf[0]);

            I finalnode = make_leaf(outleaf);

            if (mantissa != 0) {
                // Cache the result to save additional recomputation:
                pptr->value.res = finalnode;
                uint64_t new_gcdesc = ((1 + exponent) << 7) | (mantissa - 1);
                pptr->gcflags = (pptr->gcflags & 511) | (new_gcdesc << 9);
            }

            // Return the result:
            return hypernode<I>(finalnode, 0);

        } else {

            auto ch9 = ninechildren(hnode);
            if (mantissa == 0) {
                return hypernode<I>(ch9[4], hnode.depth - 1);
            }
            uint64_t newmant = bothstages ? mantissa : 0;

            for (uint64_t i = 0; i < 9; i++) {
                auto fh = iterate_recurse1(hypernode<I>(ch9[i], hnode.depth - 1), newmant, exponent);
                ch9[i] = fh.index;
            }

            auto ch4 = fourchildren(hnode, ch9);

            for (uint64_t i = 0; i < 4; i++) {
                auto fh = iterate_recurse1(hypernode<I>(ch4[i], hnode.depth - 1), mantissa, exponent);
                ch4[i] = fh.index;
            }

            I finalnode = make_nonleaf(hnode.depth - 1, ch4);

            // Cache the result to save additional recomputation:
            pptr->value.res = finalnode;
            uint64_t new_gcdesc = ((1 + exponent) << 7) | (mantissa - 1);
            pptr->gcflags = (pptr->gcflags & 511) | (new_gcdesc << 9);

            // Return the result:
            return hypernode<I>(finalnode, hnode.depth - 1);
        }
    }
};

template <typename I, typename J = lifemeta<I>>
class lifetree : public lifetree_generic<I, J> {

public:
    using lifetree_generic<I, J>::iterate_recurse;
    using lifetree_generic<I, J>::iterate_recurse1;

    lifetree(uint64_t maxmem) {
        // maxmem is specified in MiB, so we left-shift by 20:
        this->gc_threshold = maxmem << 20;
    }

    hypernode<I> iterate_recurse(hypernode<I> hnode, uint64_t mantissa, uint64_t exponent) {
        return iterate_recurse1(hnode, mantissa, exponent);
    }

    bool threshold_gc(uint64_t threshold) {

        if (this->htree.gc_partial()) {
            return true;
        }

        if (threshold) {
            uint64_t oldsize = this->htree.total_bytes();
            if (oldsize >= threshold) {
                this->htree.gc_full();
                return true;
            }
        }
        return false;
    }
};

}  // namespace apg
