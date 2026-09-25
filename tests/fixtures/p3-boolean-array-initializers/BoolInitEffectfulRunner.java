import java.lang.reflect.Method;
import java.util.Arrays;

public final class BoolInitEffectfulRunner {
    private static void run(int mode) throws Exception {
        BoolInitEffectful.reset();
        try {
            Method values = BoolInitEffectful.class.getDeclaredMethod("values", int.class);
            values.setAccessible(true);
            Object result = values.invoke(null, mode);
            String rendered = result instanceof boolean[]
                    ? Arrays.toString((boolean[]) result)
                    : Arrays.toString((int[]) result);
            System.out.println(mode + ":array=" + rendered + ":trace=" + BoolInitEffectful.trace());
        } catch (java.lang.reflect.InvocationTargetException error) {
            Throwable cause = error.getCause();
            System.out.println(mode + ":exception=" + cause.getClass().getName() + ":message="
                    + cause.getMessage() + ":trace=" + BoolInitEffectful.trace());
        }
    }

    public static void main(String[] args) throws Exception {
        run(-1);
        run(0);
        run(1);
        run(2);
        run(3);
    }
}
