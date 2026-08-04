#include <jni.h>
#include "worldgen.h"

extern "C" {

JNIEXPORT jlong JNICALL
Java_wg_WorldGen_nativeProbe(JNIEnv* /*env*/, jclass /*cls*/,
                             jlong seed, jint x, jint z) {
    return static_cast<jlong>(wg::probe(seed, x, z));
}

} // extern "C"
