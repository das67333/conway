#pragma once
#include "lifetree.h"

namespace apg {

template <typename I>
class basepattern {
    /*
     * Patterns with dynamic garbage collection.
     * A basepattern<I> can be instantiated either from a macrocell file:
     *
     * basepattern<I> x(&lt, "filename.mc");
     *
     * or from an RLE literal:
     *
     * basepattern<I> glider(&lt, "3o$o$bo!", "b3s23");
     */

    hypernode<I> hnode;
    uint64_t ihandle;
    lifetree_abstract<I> *lab;

public:
    uint64_t minp;
    uint64_t dt;
    int64_t dx;
    int64_t dy;

    lifetree_abstract<I> *getlab() const {
        return lab;
    }
    hypernode<I> gethnode() const {
        return hnode;
    }

    // We include some constructors:

    basepattern(lifetree_abstract<I> *lab, hypernode<I> hnode, int64_t dx, int64_t dy, uint64_t dt, uint64_t minp) {
        this->lab = lab;
        this->ihandle = lab->newihandle(hnode);
        this->hnode = hnode;
        this->dt = dt;
        this->minp = minp;
        this->dx = dx;
        this->dy = dy;
    }

    basepattern(lifetree_abstract<I> *lab, hypernode<I> hnode) {
        this->lab = lab;
        this->ihandle = lab->newihandle(hnode);
        this->hnode = hnode;
        this->dt = 0;
        this->minp = 0;
        this->dx = 0;
        this->dy = 0;
    }

    basepattern(lifetree_abstract<I> *lab, std::string filename) {
        hypernode<I> loaded = lab->load_macrocell(filename);
        this->lab = lab;
        this->ihandle = lab->newihandle(loaded);
        this->hnode = loaded;
        this->dt = 0;
        this->minp = 0;
        this->dx = 0;
        this->dy = 0;
    }

    // The basepattern<I> class manages resources (the associated lifetree keeps
    // a handle so that the basepattern<I> is saved from garbage-collection);
    // these need to be released when appropriate.

    basepattern(const basepattern<I> &p) {
        lab = p.getlab();
        hnode = p.gethnode();
        dx = p.dx;
        dy = p.dy;
        dt = p.dt;
        minp = p.minp;
        ihandle = lab->newihandle(hnode);
    }

    ~basepattern() {
        lab->delhandle(ihandle);
        lab->threshold_gc();
    }

    // End of resource-management code.

    // Pattern advancing:
    basepattern<I> advance(uint64_t numgens) {
        return basepattern<I>(lab, lab->advance(hnode, numgens), dx, dy, dt, minp);
    }

    // Population counts:

    uint32_t popcount(uint32_t modprime, uint64_t layermask) {
        return lab->getpop_recurse(hnode, modprime, layermask);
    }

    uint32_t popcount(uint32_t modprime) {
        return this->popcount(modprime, -1);
    }

    basepattern<I> metafy(const basepattern<I> &other, const basepattern<I> &other2) {
        uint64_t trans = 8 << other.gethnode().depth;
        basepattern<I> x = tensor(other.getchild(0), other2.getchild(0));
        x += tensor(other.getchild(1), other2.getchild(1)).shift(trans, 0);
        x += tensor(other.getchild(2), other2.getchild(2)).shift(0, trans);
        x += tensor(other.getchild(3), other2.getchild(3)).shift(trans, trans);
        return x;
    }

    void write_macrocell(std::ostream &outstream) {
        lab->write_macrocell(outstream, hnode);
    }
};

}  // namespace apg
