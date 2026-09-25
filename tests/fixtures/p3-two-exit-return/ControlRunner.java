public final class ControlRunner {
    public static void main(String[] args) throws Exception {
        Class<?> probe = Class.forName(args[0]);
        java.lang.reflect.Constructor<?> ctor = probe.getConstructor(String.class, String.class);
        java.lang.reflect.Method method = probe.getMethod("bothMatch", probe);
        String[][] inputs = {
            {null, null, null, null}, {"a", "b", "a", "b"},
            {null, "b", "a", "b"}, {"a", null, "a", "b"},
            {"a", "b", null, "b"}, {"a", "b", "a", null},
            {"x", "b", "a", "b"}, {"a", "x", "a", "b"}
        };
        for (String[] input : inputs) {
            Object left = ctor.newInstance(input[0], input[1]);
            Object right = ctor.newInstance(input[2], input[3]);
            System.out.println(method.invoke(left, right));
        }
    }
}
