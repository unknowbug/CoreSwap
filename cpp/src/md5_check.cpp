#include <cstdio>
#include "md5.h"
int main() {
    for (const char* s : {"minecraft:temperature", "octave_-10", "minecraft:continentalness"}) {
        auto h = wg::md5(std::string(s));
        printf("%s -> ", s);
        for (auto b : h) printf("%02x", b);
        printf("\n");
    }
    return 0;
}
