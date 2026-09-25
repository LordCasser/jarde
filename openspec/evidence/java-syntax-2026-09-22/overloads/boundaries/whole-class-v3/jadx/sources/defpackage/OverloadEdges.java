package defpackage;

import java.util.function.Supplier;

/* JADX INFO: loaded from: OverloadEdges.class */
public class OverloadEdges {
    public static int choose(Object obj) {
        return 1;
    }

    public static int choose(String str) {
        return 2;
    }

    public static int arr(Object obj) {
        return 3;
    }

    public static int arr(String[] strArr) {
        return 4;
    }

    public static int boxed(Object obj) {
        return 5;
    }

    public static int boxed(Integer num) {
        return 6;
    }

    public static int action(Runnable runnable) {
        return 7;
    }

    public static int action(Supplier<String> supplier) {
        return 8;
    }

    public static int nullObject() {
        return choose((Object) null);
    }

    public static int arrayObject(String[] strArr) {
        return arr((Object) strArr);
    }

    public static int boxObject(int i) {
        return boxed((Object) Integer.valueOf(i));
    }

    public static int lambdaRunnable() {
        return action(() -> {
            System.nanoTime();
        });
    }

    public static int lambdaSupplier() {
        return action((Supplier<String>) () -> {
            return "s";
        });
    }

    public static int methodRefRunnable() {
        return action(System::nanoTime);
    }
}
