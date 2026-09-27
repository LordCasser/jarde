package dt29;

import java.io.Closeable;

/* JADX INFO: loaded from: interfaces-input.jar:dt29/InterfaceCast.class */
public class InterfaceCast {
    public static Runnable asRunnable(Closeable closeable) {
        return (Runnable) closeable;
    }

    public static String choose(Closeable closeable) {
        return "closeable";
    }

    public static String choose(Runnable runnable) {
        return "runnable";
    }

    public static String chooseRunnable(Closeable closeable) {
        return choose((Runnable) closeable);
    }
}
