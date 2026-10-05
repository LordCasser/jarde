import java.util.*;
public class AW {
    static class MyErr extends Exception { }
    static class Work implements Runnable { public void run() { } }
    static void sinkT(Throwable t) { }
    static void runRunnable(Runnable r) { r.run(); }
    static void viaAbsent() {                                            // 目标 java.lang.Throwable 不在任何快照 header 上：仍拒
        sinkT(new MyErr());
    }
    static void viaNamedPlatform() {                                     // 目标 java.lang.Runnable 逐字列在 AW$Work 的 header 上：单边证明
        runRunnable(new Work());
    }
    public static void main(String[] a) { viaNamedPlatform(); }
}
