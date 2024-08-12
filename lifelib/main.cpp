#include <chrono>
#include <iostream>

#include "pattern2.h"
#include "streamlife.h"

void test(apg::lifetree_abstract<unsigned> *lt) {
    apg::basepattern<unsigned> x(lt, "res/0e0p-metaglider.mc");

    auto t1 = std::chrono::high_resolution_clock::now();

    auto y = x.advance(1 << 10);

    auto t2 = std::chrono::high_resolution_clock::now();

    std::cout << "Time: " << std::chrono::duration_cast<std::chrono::milliseconds>(t2 - t1).count() << "ms\n";

    uint32_t mod = 1000000007;
    std::cout << "Population: " << x.popcount(mod) << '\t' << y.popcount(mod) << '\n';
}

int main() {
    using I = unsigned;

    {
        apg::lifetree<I> lt(8000);
        test(&lt);
    }
    {
        apg::streamtree<I, 1> lt(8000);
        test(&lt);
    }
}
