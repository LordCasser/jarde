package defpackage;

import java.util.function.Supplier;

/* JADX INFO: loaded from: MethodRefRunnable.class */
public class MethodRefRunnable {
    public static int action(Runnable runnable) {
        return 7;
    }

    public static int action(Supplier<String> supplier) {
        return 8;
    }

    public static int run() {
        return action(System::nanoTime);
    }
}
