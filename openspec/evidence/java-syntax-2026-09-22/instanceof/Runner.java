import java.lang.reflect.Method;
public class Runner {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName(args[0]);
        Object[] inputs = { null, "abc", 7, new Object(), new int[0], new String[0], new String[0][0], (Runnable) () -> {} };
        for (String name : new String[] {"string", "number", "runnable", "primitiveArray", "referenceArray", "multiArray", "called", "local", "branch", "negated"}) {
            Method method = type.getMethod(name, Object.class);
            for (int i = 0; i < inputs.length; i++) {
                type.getField("calls").setInt(null, 0);
                System.out.println(name + "/" + i + "=" + method.invoke(null, inputs[i]) + ":" + type.getField("calls").getInt(null));
            }
        }
    }
}
