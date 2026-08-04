#include <cstdio>
#include <cstdint>
#include "random.h"
#include "xoroshiro.h"
using namespace wg;
int main() {
    uint64_t seed = 0x8D5F04C9C2F7B6E2ULL; // -8248318472910187742 ???
    auto s = createXoroshiroSeed(seed);
    printf("lo=%lld hi=%lld\n", (long long)s.seedLo, (long long)s.seedHi);
    Xoroshiro128PlusPlus r(s.seedLo, s.seedHi);
    uint64_t n1 = r.next();
    uint64_t n2 = r.next();
    printf("next1=%lld next2=%lld\n", (long long)n1, (long long)n2);
    // ?????
    Xoroshiro128PlusPlus r2(seed);
    printf("ctor next1=%lld next2=%lld\n", (long long)r2.next(), (long long)r2.next());
    return 0;
}
