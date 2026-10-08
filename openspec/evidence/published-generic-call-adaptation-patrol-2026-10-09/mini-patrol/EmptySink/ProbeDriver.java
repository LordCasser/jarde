import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class ProbeDriver {
    private static void describe(String label, Method method) {
        System.out.println(label + ".genericParameters=" + java.util.Arrays.toString(method.getGenericParameterTypes()));
        System.out.println(label + ".genericReturn=" + method.getGenericReturnType());
        System.out.println(label + ".erasedParameters=" + java.util.Arrays.toString(method.getParameterTypes()));
        System.out.println(label + ".erasedReturn=" + method.getReturnType().getTypeName());
    }
    public static void main(String[] args) throws Exception {
        String probe = args[0];
        Class<?> type = Class.forName(args[1]);
        Object marker = new Object();
        if (probe.equals("EmptySink")) {
            describe("sink", type.getMethod("sink", Object.class));
        } else if (probe.equals("FieldSetter")) {
            Field field = type.getField("value");
            Method set = type.getMethod("set", Object.class);
            System.out.println("field.generic=" + field.getGenericType());
            describe("set", set);
            Object instance = type.getConstructor().newInstance();
            set.invoke(instance, marker);
            System.out.println("behavior.field-is-marker=" + (field.get(instance) == marker));
        } else if (probe.equals("CallRelay")) {
            describe("identity", type.getMethod("identity", Object.class));
            Method relay = type.getMethod("relay", Object.class);
            describe("relay", relay);
            Object instance = type.getConstructor().newInstance();
            System.out.println("behavior.relay-is-marker=" + (relay.invoke(instance, marker) == marker));
        } else if (probe.equals("TypedSetter")) {
            Field field = type.getField("value");
            Method set = type.getMethod("set", type, Object.class);
            System.out.println("field.generic=" + field.getGenericType());
            describe("set", set);
            Object instance = type.getConstructor().newInstance();
            set.invoke(instance, instance, marker);
            System.out.println("behavior.field-is-marker=" + (field.get(instance) == marker));
        } else {
            throw new IllegalArgumentException(probe);
        }
    }
}
