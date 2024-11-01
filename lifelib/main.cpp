#include <chrono>
#include <iostream>

#include "pattern2.h"
#include "streamlife.h"

void test(apg::lifetree_abstract<unsigned> *lt, int steps_log2) {
    apg::basepattern<unsigned> x(lt, "res/0e0p-metaglider.mc");

    auto t1 = std::chrono::high_resolution_clock::now();

    auto y = x.advance(1ul << steps_log2);

    auto t2 = std::chrono::high_resolution_clock::now();

    uint32_t mod = 1000000007;
    std::cout << "steps_log2=" << steps_log2
        << "\tpopulation=" << y.popcount(mod)
        << "\thash=" << y.hash()
        << "\ttime=" << std::chrono::duration<double>(t2 - t1).count()
        << std::endl;
}

int main() {
    using I = unsigned;

    for (int steps_log2 = 29; steps_log2 <= 36; steps_log2++) {
        //     apg::lifetree<I> lt(16000);
        //     test(&lt, steps_log2);
        apg::streamtree<I, 1> lt(1'000'000);
        test(&lt, steps_log2);
    }
}
