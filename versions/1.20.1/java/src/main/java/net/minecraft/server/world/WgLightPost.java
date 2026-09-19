// 260919-09（design-260919-08 B1 主攻包 FA-1）：POST_UPDATE 段投递助手。普通类（非 mixin；无 static
// nested class 问题）。经由 coreswap.accesswidener 加宽的公有 enqueue(int,int,Stage,Runnable) 直接投递
// ——1:1 复刻 vanilla light() 尾部（yarn SLP.java:179-183）；runTasks 阶段序 = PRE 批 → doLightUpdates
// → POST 批（SLP.java:195-218），保证「section 数据已入库且传播已消化」先于终态化。
// 历史：@Invoker 方案被编译否决（mapped jar 中 Stage 对非 nest-mate = private，260919-09 compileJava）。
package net.minecraft.server.world;

public final class WgLightPost {
    private WgLightPost() {}

    public static void postUpdate(ServerLightingProvider slp, int x, int z, Runnable task) {
        slp.enqueue(x, z, ServerLightingProvider.Stage.POST_UPDATE, task);
    }
}
