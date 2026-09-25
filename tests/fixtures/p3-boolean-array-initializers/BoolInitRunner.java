import java.lang.reflect.Method;
import java.util.Arrays;

public final class BoolInitRunner {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName("BoolInit");
        Method values = type.getDeclaredMethod("values");
        values.setAccessible(true);
        Object result = values.invoke(null);
        if (result instanceof boolean[]) {
            System.out.println(Arrays.toString((boolean[]) result));
        } else {
            System.out.println(Arrays.toString((int[]) result));
        }
    }
}
