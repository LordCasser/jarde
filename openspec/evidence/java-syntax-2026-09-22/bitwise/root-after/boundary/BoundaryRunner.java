import java.lang.reflect.Method;

public final class BoundaryRunner {
    private static Object call(Class<?> owner, String name, Class<?>[] types, Object... args) throws Exception {
        Method method = owner.getMethod(name, types);
        return method.invoke(null, args);
    }

    public static void main(String[] args) throws Exception {
        boolean patched = args.length != 0 && args[0].equals("patched");
        String binaryName = args.length > 1 ? args[1] : "BoundaryProbe";
        Class<?> owner = Class.forName(binaryName);
        System.out.println("pure-int-01=" + call(owner, "pureIntegerLiterals",
                new Class<?>[] { int.class, int.class }, 5, 2));
        System.out.println("overwritten-local=" + call(owner, "overwrittenLocal",
                new Class<?>[] { int.class, int.class }, 6, 3));
        System.out.println("dup-call-pop=" + call(owner, "observeOnce",
                new Class<?>[] { int.class, int.class }, 6, 3));

        if (patched) {
            for (boolean left : new boolean[] { false, true }) {
                for (int right : new int[] { 0, 1, 2, -1 }) {
                    System.out.println("ZI:" + left + ":" + right + "=" + call(owner, "mixedLeft",
                            new Class<?>[] { boolean.class, int.class, long.class }, left, right, 0L));
                }
            }
            for (int left : new int[] { 0, 1, 2, -1 }) {
                for (boolean right : new boolean[] { false, true }) {
                    System.out.println("IZ:" + left + ":" + right + "=" + call(owner, "mixedRight",
                            new Class<?>[] { int.class, boolean.class, byte.class }, left, right, (byte) 0));
                }
            }
        } else {
            System.out.println("source-int-left=" + call(owner, "mixedLeft",
                    new Class<?>[] { int.class, int.class, long.class }, 6, 3, 0L));
            System.out.println("source-int-right=" + call(owner, "mixedRight",
                    new Class<?>[] { int.class, int.class, byte.class }, 6, 3, (byte) 0));
        }
        System.out.println("trace=" + owner.getField("trace").getInt(null));
    }
}
