// noise_direct_probe.cpp —— D23：CpuBackend 直接采样 noodle_ridge_b vs sim 拆分采样
// 对 (784,160,-408) corner0：CpuBackend.normals[vi].sample(坐标链) 直接采样，
// 对比 sim（拆分采样）的 normal_noise(192)=-0.0165。定位拆分/采样逻辑分叉。
#include <cstdio>
#include <cstdint>
#include <cmath>
#include "cpu_backend.h"

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    CpuBackend backend;
    backend.init(worldSeed);
    const int px = 784, py = 160, pz = -408;
    // noodle_ridge_b@c0 = normals[184]（纯 normal 序号）
    // 坐标链：noodle_ridge_b 的 coord_chains（shifted_noise + flat_cache）
    // 从 dump_noise_layout：noodle_ridge_b 的 chain 需查——先打印 184 的 chain
    // 直接采样（模拟 splitDouble 的坐标公式：x*scale + shift）
    // 先尝试 plain sample（scale=2.6666667, y_scale=2.6666667）
    for (int vi : {184, 176, 168, 160}) {
        double x = px * 2.6666666666666665;
        double y = py * 2.6666666666666665;
        double z = pz * 2.6666666666666665;
        double v = backend.normals[vi].sample(x, y, z);
        std::printf("normals[%d].sample(%.2f,%.2f,%.2f) = %.9f\n", vi, x, y, z, v);
    }
    // D23：spagrough@c0（实例 56，纯 normal 56）scale=1.0 直接采样 vs GPU 拆分采样
    {
        double v = backend.normals[56].sample((double)px, (double)py, (double)pz);
        std::printf("normals[56] spagrough@c0 direct sample(%d,%d,%d) = %.9f (GPU node54=-0.113109)\n", px, py, pz, v);
    }
    // 拆分采样模拟（读 split 数据：sim 的 normal_noise 逻辑）
    std::vector<float> sc((size_t)backend.splitTotal);
    backend.split(px, py, pz, sc.data());
    // noodle_ridge_b@c0 splitBase=8576，n=1，octBase=1376
    for (int base : {8576, 8480, 8384, 8288}) {
        int ix = (int)sc[base+0], iy = (int)sc[base+1], iz = (int)sc[base+2];
        float gx = sc[base+3], gy = sc[base+4], gz = sc[base+5];
        std::printf("split[%d]: (%d,%d,%d) + (%.4f,%.4f,%.4f)\n", base, ix, iy, iz, gx, gy, gz);
    }
    // D23：spagrough@c0 splitBase=6080（实例 56）——拆分坐标 vs 直接采样
    {
        int base = 6080;
        int ix = (int)sc[base+0], iy = (int)sc[base+1], iz = (int)sc[base+2];
        float gx = sc[base+3], gy = sc[base+4], gz = sc[base+5];
        std::printf("roughness@c0 split[%d]: (%d,%d,%d) + (%.4f,%.4f,%.4f)\n", base, ix, iy, iz, gx, gy, gz);
        // normals[48] = roughness（vi=48, scale=1）直接采样
        double v48 = backend.normals[48].sample((double)px, (double)py, (double)pz);
        std::printf("normals[48] roughness direct sample(%d,%d,%d) = %.9f (sim node54=-0.113109)\n", px, py, pz, v48);
        // 拆分采样手动（用 split 数据 + perm）
        std::printf("  -> 拆分采样应 ≈ 直接采样 0.4159? 或 -0.113?（node54 是 roughness 还是 rarity 待定）\n");
    }
    // D23：continentalness@c0（实例 0）——sim normal_noise(0)=0.0602 vs 直接采样
    // chain: shifted_noise flat_cache, xz_scale=0.25, shift_x=offset@(x*0.25,0,z*0.25)*4
    {
        // 拆分采样（sim 方式）：读 splitCoord[0..]（实例 0 splitBase=0）
        int base = 0;
        int ix = (int)sc[base+0], iy = (int)sc[base+1], iz = (int)sc[base+2];
        float gx = sc[base+3], gy = sc[base+4], gz = sc[base+5];
        std::printf("continentalness@c0 split[%d]: (%d,%d,%d) + (%.4f,%.4f,%.4f)\n", base, ix, iy, iz, gx, gy, gz);
        // 直接采样（模拟 chain：flat_cache 对齐 (x>>2)<<2, 0, (z>>2)<<2，scale 0.25 + offset shift）
        double ax = ((px >> 2) << 2) * 0.25;
        double az = ((pz >> 2) << 2) * 0.25;
        double sx = backend.shiftNoises.at("minecraft:offset").sample(ax, 0.0, az) * 4.0;
        double sz = backend.shiftNoises.at("minecraft:offset").sample(az, ax, 0.0) * 4.0;
        double dx = ax + sx, dz = az + sz;
        double v = backend.normals[0].sample(dx, 0.0, dz);
        std::printf("continentalness@c0 direct sample(flat_cache chain) = %.9f (sim=0.0602)\n", v);
    }
    return 0;
}
