#pragma once

#include "lifetree.h"

namespace apg {

template <typename I>
struct streammeta {

    I res;
    I aux;
    uint64_t lanes;
};

template <typename I, int N>
class streamtree : public lifetree_generic<I, streammeta<I>> {

public:
    using lifetree_generic<I, streammeta<I>>::htree;
    using lifetree_generic<I, streammeta<I>>::iterate_recurse;
    using lifetree_generic<I, streammeta<I>>::iterate_recurse1;
    using lifetree_generic<I, streammeta<I>>::ind2ptr_leaf;
    using lifetree_generic<I, streammeta<I>>::ind2ptr_nonleaf;
    using lifetree_generic<I, streammeta<I>>::ninechildren;
    using lifetree_generic<I, streammeta<I>>::fourchildren;
    using lifetree_generic<I, streammeta<I>>::make_nonleaf;
    using lifetree_abstract<I>::breach;

    streamtree(uint64_t maxmem) {
        // maxmem is specified in MiB, so we left-shift by 20:
        this->gc_threshold = maxmem << 20;
    }

    /*
     * Streamlife operates on pairs of hashtiles instead of individual
     * hashtiles. We want to memoize the results of these computations.
     *
     * Key:   nicearray<I, 4> = (beszel_tile,  ulqoma_tile, depth, gencount)
     * Value: nicearray<I, 2> = (beszel_tile', ulqoma_tile')
     */
    kivtable<nicearray<I, 4>, I, nicearray<I, 2>> biresults;

    bool threshold_gc(uint64_t threshold) {

        if (this->htree.gc_partial()) {
            // We have invalidated biresults, so we empty it:
            biresults.clear();
            return true;
        }

        if (threshold) {
            uint64_t oldsize = this->htree.total_bytes() + biresults.total_bytes();
            if (oldsize >= threshold) {
                std::cerr << "Emptying streamlife biresults (" << oldsize << " >= " << threshold << ")" << std::endl;
                biresults.clear();
                uint64_t newsize = this->htree.total_bytes() + biresults.total_bytes();
                if (newsize >= ((threshold * 3) / 4)) {
                    std::cerr << "Insufficient memory liberated; entering full garbage "
                                 "collection..."
                              << std::endl;
                    this->htree.gc_full();
                    newsize = this->htree.total_bytes() + biresults.total_bytes();
                }
                std::cerr << "Streamlife size reduced from " << oldsize << " to " << newsize << " bytes." << std::endl;
                return true;
            }
        }
        return false;
    }

    uint64_t node2lanes(I depth, I index) {

        if (index == 0) {
            // std::cerr << "index = 0" << std::endl;
            return 0xffff;
        }

        if (depth == 0) {
            auto pptr = this->ind2ptr_leaf(index);
            if ((pptr->value.lanes & 0xffff0000ull) != (1 << 16)) {
                pptr->value.lanes = determine_direction(&pptr->key[0]) | (1 << 16);
            }
            /*
            if (pptr->value.lanes & 255) {
                std::cout << (pptr->value.lanes & 255) << std::endl;
                printnode(hypernode<I>(index, depth));
            }
            */
            // std::cerr << pptr->value.lanes << std::endl;
            return pptr->value.lanes & 0xffffffff0000ffffull;
        } else {
            auto pptr = this->ind2ptr_nonleaf(depth, index);
            if ((pptr->value.lanes & 0xffff0000ull) != (1 << 16)) {

                uint64_t childlanes[9];
                uint64_t adml = 0xff;

                /*
                 * Short-circuit evaluation using the corner children.
                 * This will handle the vast majority of random tiles.
                 */
                if (adml != 0) {
                    childlanes[0] = node2lanes(depth - 1, pptr->key[0]);
                    adml &= childlanes[0];
                }
                if (adml != 0) {
                    childlanes[2] = node2lanes(depth - 1, pptr->key[1]);
                    adml &= childlanes[2];
                }
                if (adml != 0) {
                    childlanes[6] = node2lanes(depth - 1, pptr->key[2]);
                    adml &= childlanes[6];
                }
                if (adml != 0) {
                    childlanes[8] = node2lanes(depth - 1, pptr->key[3]);
                    adml &= childlanes[8];
                }
                if (adml == 0) {
                    pptr->value.lanes = (1 << 16);
                    return 0;
                }

                if (depth == 1) {
                    uint64_t *tlx = &ind2ptr_leaf(pptr->key[0])->key[0];
                    uint64_t *trx = &ind2ptr_leaf(pptr->key[1])->key[0];
                    uint64_t *blx = &ind2ptr_leaf(pptr->key[2])->key[0];
                    uint64_t *brx = &ind2ptr_leaf(pptr->key[3])->key[0];

                    nicearray<uint64_t, 4 * N> cc, tc, bc, cl, cr;

                    for (uint64_t i = 0; i < N; i++) {
                        uint64_t arr[20] = {tlx[3], trx[2], blx[1], brx[0], tlx[1], trx[0], tlx[3],
                                            trx[2], blx[1], brx[0], blx[3], brx[2], tlx[2], tlx[3],
                                            blx[0], blx[1], trx[2], trx[3], brx[0], brx[1]};

                        std::memcpy(&cc[4 * i], arr, 32);
                        std::memcpy(&tc[4 * i], arr + 4, 32);
                        std::memcpy(&bc[4 * i], arr + 8, 32);
                        std::memcpy(&cl[4 * i], arr + 12, 32);
                        std::memcpy(&cr[4 * i], arr + 16, 32);
                        tlx += 4;
                        trx += 4;
                        blx += 4;
                        brx += 4;
                    }

                    childlanes[1] = node2lanes(depth - 1, this->make_leaf(tc));
                    childlanes[3] = node2lanes(depth - 1, this->make_leaf(cl));
                    childlanes[4] = node2lanes(depth - 1, this->make_leaf(cc));
                    childlanes[5] = node2lanes(depth - 1, this->make_leaf(cr));
                    childlanes[7] = node2lanes(depth - 1, this->make_leaf(bc));
                    adml &= (childlanes[1] & childlanes[3] & childlanes[4] & childlanes[5] & childlanes[7]);
                } else {
                    auto pptr_tl = ind2ptr_nonleaf(depth - 1, pptr->key[0]);
                    auto pptr_tr = ind2ptr_nonleaf(depth - 1, pptr->key[1]);
                    auto pptr_bl = ind2ptr_nonleaf(depth - 1, pptr->key[2]);
                    auto pptr_br = ind2ptr_nonleaf(depth - 1, pptr->key[3]);
                    nicearray<I, 4> cc = {pptr_tl->key[3], pptr_tr->key[2], pptr_bl->key[1], pptr_br->key[0]};
                    nicearray<I, 4> tc = {pptr_tl->key[1], pptr_tr->key[0], pptr_tl->key[3], pptr_tr->key[2]};
                    nicearray<I, 4> bc = {pptr_bl->key[1], pptr_br->key[0], pptr_bl->key[3], pptr_br->key[2]};
                    nicearray<I, 4> cl = {pptr_tl->key[2], pptr_tl->key[3], pptr_bl->key[0], pptr_bl->key[1]};
                    nicearray<I, 4> cr = {pptr_tr->key[2], pptr_tr->key[3], pptr_br->key[0], pptr_br->key[1]};
                    childlanes[1] = node2lanes(depth - 1, make_nonleaf(depth - 1, tc));
                    childlanes[3] = node2lanes(depth - 1, make_nonleaf(depth - 1, cl));
                    childlanes[4] = node2lanes(depth - 1, make_nonleaf(depth - 1, cc));
                    childlanes[5] = node2lanes(depth - 1, make_nonleaf(depth - 1, cr));
                    childlanes[7] = node2lanes(depth - 1, make_nonleaf(depth - 1, bc));
                    adml &= (childlanes[1] & childlanes[3] & childlanes[4] & childlanes[5] & childlanes[7]);
                }
                /*
                if ((adml == 4) && (depth <= 3)) {
                    std::cout << (adml) << std::endl;
                    printnode(hypernode<I>(index, depth));
                }
                */

                for (uint64_t i = 0; i < 9; i++) {
                    childlanes[i] >>= 32;
                }
                uint64_t lanes = 0;

#define ROTR32(X, Y) (((X) >> (Y)) | ((X) << (32 - (Y))))
#define ROTL32(X, Y) (((X) << (Y)) | ((X) >> (32 - (Y))))

                /*
                 * Lane numbers are modulo 32, with each lane being either
                 * 8 rows, 8 columns, or 8hd (in either diagonal direction)
                 */
                uint64_t a = (depth < 6) ? (1 << (depth - 1)) : 0;
                uint64_t a2 = (2 * a) & 31;

                if (adml & 0x88) {
                    // Horizontal lanes
                    lanes |= ROTL32(childlanes[0] | childlanes[1] | childlanes[2], a);
                    lanes |= (childlanes[3] | childlanes[4] | childlanes[5]);
                    lanes |= ROTR32(childlanes[6] | childlanes[7] | childlanes[8], a);
                }

                if (adml & 0x44) {
                    lanes |= ROTL32(childlanes[0], a2);
                    lanes |= ROTL32(childlanes[3] | childlanes[1], a);
                    lanes |= (childlanes[6] | childlanes[4] | childlanes[2]);
                    lanes |= ROTR32(childlanes[7] | childlanes[5], a);
                    lanes |= ROTR32(childlanes[8], a2);
                }

                if (adml & 0x22) {
                    // Vertical lanes
                    lanes |= ROTL32(childlanes[0] | childlanes[3] | childlanes[6], a);
                    lanes |= (childlanes[1] | childlanes[4] | childlanes[7]);
                    lanes |= ROTR32(childlanes[2] | childlanes[5] | childlanes[8], a);
                }

                if (adml & 0x11) {
                    lanes |= ROTL32(childlanes[2], a2);
                    lanes |= ROTL32(childlanes[1] | childlanes[5], a);
                    lanes |= (childlanes[0] | childlanes[4] | childlanes[8]);
                    lanes |= ROTR32(childlanes[3] | childlanes[7], a);
                    lanes |= ROTR32(childlanes[6], a2);
                }

                pptr->value.lanes = adml | (1 << 16) | (lanes << 32);
            }
            return pptr->value.lanes & 0xffffffff0000ffffull;
        }
    }

    uint64_t is_solitonic(hypernode<I> hnode) {

        uint64_t lanes1 = node2lanes(hnode.depth, hnode.index);
        if ((lanes1 & 255) == 0) {
            return 0;
        }
        uint64_t lanes2 = node2lanes(hnode.depth, hnode.index2);
        if ((lanes2 & 255) == 0) {
            return 0;
        }
        uint64_t commonlanes = (lanes1 & lanes2) >> 32;
        if (commonlanes) {
            return 0;
        }
        return ((((lanes1 >> 4) & lanes2) | ((lanes2 >> 4) & lanes1)) & 15);
    }

    hypernode<I> iterate_recurse(hypernode<I> hnode, uint64_t mantissa, uint64_t exponent) {

        hypernode<I> part1(hnode.index, hnode.depth);
        hypernode<I> part2(hnode.index2, hnode.depth);

        if (is_solitonic(hnode)) {
            // BESZEL and ULQOMA tiles are provably non-interacting:
            // std::cerr << "is solitonic" << std::endl;
            I i1 = iterate_recurse1(part1, mantissa, exponent).index;
            I i2 = iterate_recurse1(part2, mantissa, exponent).index;

            if ((hnode.index == 0) || (hnode.index2 == 0)) {
                I i3 = i1 | i2;
                I ind3 = hnode.index | hnode.index2;
                auto lanes = node2lanes(hnode.depth, ind3);
                if (lanes & 240) {
                    return hypernode<I>(0, i3, hnode.depth - 1);
                } else {
                    return hypernode<I>(i3, 0, hnode.depth - 1);
                }
            } else {
                return hypernode<I>(i1, i2, hnode.depth - 1);
            }

        } else {
            uint64_t effexp = (hnode.depth < (1 + exponent)) ? hnode.depth : (1 + exponent);
            I gcdesc = (effexp << 7) | (mantissa - 1);
            nicearray<I, 4> k = {hnode.index, hnode.index2, hnode.depth, gcdesc};

            I p = biresults.getnode(k, false);
            if (p == ((I)-1)) {

                hypernode<I> res(0, 0, hnode.depth - 1);

                if (hnode.depth == 1) {
                    hypernode<I> hnode2 = breach(hnode);
                    I i3 = iterate_recurse1(hnode2, mantissa, exponent).index;

                    if (i3 != 0) {
                        uint64_t lanes = node2lanes(hnode2.depth, hnode2.index);
                        // if (lanes & 255) { std::cerr << lanes << std::endl; }
                        if (lanes & 240) {
                            // std::cerr << "lanes & 240" << std::endl;
                            res.index2 = i3;
                        } else {
                            res.index = i3;
                        }
                    }
                } else {

                    auto ch91 = ninechildren(part1);
                    auto ch92 = ninechildren(part2);

                    if (mantissa == 0) {
                        res.index = ch91[4];
                        res.index2 = ch92[4];
                    } else {
                        bool bothstages = (hnode.depth <= (1 + exponent));
                        uint64_t newmant = bothstages ? mantissa : 0;
                        for (uint64_t i = 0; i < 9; i++) {
                            auto fh =
                                iterate_recurse(hypernode<I>(ch91[i], ch92[i], hnode.depth - 1), newmant, exponent);
                            ch91[i] = fh.index;
                            ch92[i] = fh.index2;
                        }

                        auto ch41 = fourchildren(part1, ch91);
                        auto ch42 = fourchildren(part2, ch92);

                        for (uint64_t i = 0; i < 4; i++) {
                            auto fh = iterate_recurse(hypernode<I>(ch41[i], ch42[i], hnode.depth - 1), mantissa,
                                                      exponent);
                            ch41[i] = fh.index;
                            ch42[i] = fh.index2;
                        }

                        res.index = make_nonleaf(hnode.depth - 1, ch41);
                        res.index2 = make_nonleaf(hnode.depth - 1, ch42);
                    }
                }

                nicearray<I, 2> v = {res.index, res.index2};
                p = biresults.setnode(k, v);
            }

            auto xptr = biresults.ind2ptr(p);
            return hypernode<I>(xptr->value[0], xptr->value[1], hnode.depth - 1);
        }
    }
};

}  // namespace apg
