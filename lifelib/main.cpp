#include <chrono>
#include <iostream>

#include "pattern2.h"
#include "streamlife.h"

void test(apg::lifetree_abstract<unsigned> *lt, int steps_log2) {
    apg::basepattern<unsigned> x(lt, "res/0e0p-metaglider.mc");

    auto t1 = std::chrono::high_resolution_clock::now();

    auto y = x.advance(1ul << steps_log2);

    auto t2 = std::chrono::high_resolution_clock::now();

    std::cout << "Time: " << std::chrono::duration_cast<std::chrono::milliseconds>(t2 - t1).count() << "ms\n";

    uint32_t mod = 1000000007;
    std::cout << "Population: " << x.popcount(mod) << '\t' << y.popcount(mod) << '\n';
}

int main() {
    using I = unsigned;

    for (int steps_log2 = 0; steps_log2 <= 19; steps_log2++) {
        //     apg::lifetree<I> lt(16000);
        //     test(&lt, steps_log2);
        apg::streamtree<I, 1> lt(16000);
        test(&lt, steps_log2);
    }
}
