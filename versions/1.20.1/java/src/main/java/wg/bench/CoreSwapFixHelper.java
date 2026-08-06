package wg.bench;

import java.io.IOException;
import java.io.InputStream;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Comparator;
import java.util.Enumeration;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.stream.Stream;

/**
 * CoreSwap fix helper (3rd round).
 *
 * 原版用 FabricLoader.getModContainer("...").getRootPaths() 遍历 mod 资源，
 * 在 Sinytra Connector（Forge 宿主）环境下：Forge 的 SecureJar/UnionFileSystem
 * 返回的 root Path 无法被 Files.isDirectory / Files.walk 正常解析；
 * 而 getProtectionDomain().getCodeSource().getLocation() 对 Forge
 * TransformingClassLoader 加载的类返回 "/"（无具体 jar）。
 *
 * 本类改为多级定位「包含资源的 jar」并直接用 JarFile 提取：
 *   1. getCodeSource()（纯 Fabric 环境）
 *   2. FabricLoader.getAllMods() → ModOrigin.getPaths()（Forge+Connector，
 *      返回 ModFile.getFilePath() 的磁盘 jar 路径；反射调用避免编译依赖）
 *   3. ClassLoader.getResources("worldgen-data") 资源枚举兜底
 */
public final class CoreSwapFixHelper {
    private CoreSwapFixHelper() {
    }

    /** 替换 wg.bench.CppBridge.extractWorldgenDir() 的调用目标。 */
    public static String extractWorldgenDir() {
        String tmpDir = System.getProperty("java.io.tmpdir");
        Path target = Path.of(tmpDir, "coreswap-data");
        Path wgDir = target.resolve("worldgen");
        Path marker = wgDir.resolve("data/minecraft/worldgen/noise_settings/overworld.json");
        try {
            if (!Files.exists(marker)) {
                if (Files.exists(target)) {
                    deleteRecursively(target);
                }
                Files.createDirectories(wgDir);
                extractFromJar("worldgen-data", target, wgDir);
                if (!Files.exists(marker)) {
                    throw new IllegalStateException("worldgen-data not found in mod resources");
                }
            }
            return wgDir.toString();
        }
        catch (RuntimeException e) {
            throw e;
        }
        catch (Exception e) {
            throw new RuntimeException("failed to extract worldgen-data", e);
        }
    }

    /** 替换 wg.CppWorldgen.extractNativeDll() 的调用目标。 */
    public static String extractNativeDll() {
        String tmpDir = System.getProperty("java.io.tmpdir");
        Path dir = Path.of(tmpDir, "coreswap-native");
        Path dll = dir.resolve("worldgen.dll");
        try {
            if (!Files.exists(dll)) {
                Files.createDirectories(dir);
                extractFromJar("native", dir, dir);
            }
            if (!Files.exists(dll)) {
                throw new IllegalStateException("worldgen.dll not found in mod native/");
            }
            return dll.toString();
        }
        catch (IOException e) {
            throw new RuntimeException("failed to extract worldgen.dll", e);
        }
    }

    /**
     * 从定位到的 jar 提取 prefix/ 下的所有文件。
     * 布局与原版一致：rel 以 "data" 开头 → wgDir 下；否则 → target 下。
     */
    private static void extractFromJar(String prefix, Path target, Path wgDir) throws IOException {
        Path src = locateResource(prefix);
        if (src == null) {
            throw new IOException("cannot locate mod resource for extraction (tried codeSource, ModOrigin, classloader): " + prefix);
        }
        if (Files.isRegularFile(src)) {
            try (JarFile jf = new JarFile(src.toFile())) {
                Enumeration<JarEntry> en = jf.entries();
                while (en.hasMoreElements()) {
                    JarEntry e = en.nextElement();
                    if (e.isDirectory()) continue;
                    String name = e.getName();
                    if (!name.startsWith(prefix + "/")) continue;
                    String rel = name.substring(prefix.length() + 1);
                    Path dst = rel.startsWith("data") ? wgDir.resolve(rel) : target.resolve(rel);
                    if (dst.getParent() != null) {
                        Files.createDirectories(dst.getParent());
                    }
                    try (InputStream in = jf.getInputStream(e)) {
                        Files.copy(in, dst, StandardCopyOption.REPLACE_EXISTING);
                    }
                }
            }
        } else {
            // dev 环境（classpath 目录）：src 为 prefix 根，或含 prefix/ 的目录
            Path prefixDir = Files.isDirectory(src.resolve(prefix)) ? src.resolve(prefix) : src;
            try (Stream<Path> stream = Files.walk(prefixDir)) {
                stream.filter(Files::isRegularFile).forEach(p -> {
                    String rel = prefixDir.relativize(p).toString().replace('\\', '/');
                    Path dst = rel.startsWith("data") ? wgDir.resolve(rel) : target.resolve(rel);
                    try {
                        if (dst.getParent() != null) Files.createDirectories(dst.getParent());
                        Files.copy(p, dst, StandardCopyOption.REPLACE_EXISTING);
                    } catch (IOException e) {
                        throw new RuntimeException(e);
                    }
                });
            }
        }
    }

    /** 多级定位包含 prefix 资源的 jar 或目录（dev classpath）。 */
    private static Path locateResource(String prefix) {
        // 1) code source（纯 Fabric 环境；dev 是 classpath 目录）
        try {
            URL loc = CoreSwapFixHelper.class.getProtectionDomain().getCodeSource().getLocation();
            Path p = toPath(loc);
            if (p != null && hasResource(p, prefix)) {
                return p;
            }
        }
        catch (Exception ignored) {
        }
        // 2) FabricLoader mods → ModOrigin.getPaths()（Forge+Connector 返回磁盘 jar 路径）
        try {
            Class<?> loaderCls = Class.forName("net.fabricmc.loader.api.FabricLoader");
            Object loader = loaderCls.getMethod("getInstance").invoke(null);
            Object mods = loader.getClass().getMethod("getAllMods").invoke(loader); // Collection<ModContainer>
            for (Object mc : (Iterable<?>) mods) {
                Object origin = mc.getClass().getMethod("getOrigin").invoke(mc);    // ModOrigin
                Object paths = origin.getClass().getMethod("getPaths").invoke(origin); // List<Path>
                for (Object o : (Iterable<?>) paths) {
                    Path p = (Path) o;
                    if (hasResource(p, prefix)) {
                        return p;
                    }
                }
            }
        }
        catch (Exception ignored) {
        }
        // 3) classloader 资源枚举兜底（dev 目录 / jar）
        try {
            Enumeration<URL> urls = CoreSwapFixHelper.class.getClassLoader().getResources(prefix);
            while (urls.hasMoreElements()) {
                Path p = toPath(urls.nextElement());
                if (p != null) {
                    return p; // 目录（prefix 根）或 jar
                }
            }
        }
        catch (Exception ignored) {
        }
        return null;
    }

    /** 路径是否包含 prefix 资源（jar 或目录）。 */
    private static boolean hasResource(Path p, String prefix) {
        if (p == null) return false;
        if (Files.isRegularFile(p)) {
            return jarContains(p, prefix);
        }
        if (Files.isDirectory(p)) {
            return Files.isDirectory(p.resolve(prefix)) || (p.getFileName() != null && p.getFileName().toString().equals(prefix));
        }
        return false;
    }

    /** 快速检查 jar 是否包含某前缀资源。 */
    private static boolean jarContains(Path jarPath, String prefix) {
        try (JarFile jf = new JarFile(jarPath.toFile())) {
            Enumeration<JarEntry> en = jf.entries();
            while (en.hasMoreElements()) {
                if (en.nextElement().getName().startsWith(prefix + "/")) {
                    return true;
                }
            }
        }
        catch (IOException ignored) {
        }
        return false;
    }

    /** URL → 磁盘 Path；兼容 jar:file:/...!/ 形式。 */
    private static Path toPath(URL loc) {
        if (loc == null) return null;
        try {
            String s = loc.toString();
            if (s.startsWith("jar:")) {
                int idx = s.indexOf("!/");
                if (idx >= 0) s = s.substring(4, idx);
                loc = new URL(s);
            }
            return Path.of(loc.toURI());
        }
        catch (Exception e) {
            return null;
        }
    }

    private static void deleteRecursively(Path path) throws IOException {
        if (!Files.exists(path)) return;
        try (Stream<Path> stream = Files.walk(path)) {
            stream.sorted(Comparator.reverseOrder()).forEach(p -> {
                try {
                    Files.deleteIfExists(p);
                }
                catch (IOException ignored) {
                    // best effort
                }
            });
        }
    }
}
