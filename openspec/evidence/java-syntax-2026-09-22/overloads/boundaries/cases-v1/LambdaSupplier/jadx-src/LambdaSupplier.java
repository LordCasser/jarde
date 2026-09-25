
import java.util.function.Supplier;

/* JADX INFO: loaded from: LambdaSupplier.class */
public class LambdaSupplier {
    public static int action(Runnable runnable) {
        return 7;
    }

    public static int action(Supplier<String> supplier) {
        return 8;
    }

    public static int run() {
        return action((Supplier<String>) () -> {
            return "s";
        });
    }
}
