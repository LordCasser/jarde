import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Collections;

public class PinnedTestRunner {
    private static int run(String className, java.util.List<Object> values) throws Exception {
        Class<?> cls = Class.forName(className);
        Constructor<?> constructor = cls.getDeclaredConstructor();
        constructor.setAccessible(true);
        Object instance = constructor.newInstance();
        Method test = cls.getMethod("test", java.util.List.class);
        test.invoke(instance, values);
        Field count = cls.getDeclaredField("count");
        count.setAccessible(true);
        return count.getInt(instance);
    }

    public static void main(String[] args) throws Exception {
        String className = args[0];
        System.out.println("two:" + run(className, Arrays.<Object>asList("1", "2")));
        System.out.println("empty:" + run(className, Collections.<Object>emptyList()));
    }
}
