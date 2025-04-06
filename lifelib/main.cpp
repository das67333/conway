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
    for (int steps_log2 = 10; steps_log2 <= 10; steps_log2++) {
        // apg::lifetree<uint32_t> lt(16'000);
        apg::streamtree<uint32_t, 1> lt(16'000);
        test(&lt, steps_log2);
    }
}
