public class NonzeroHeldUse {
    static Object pick(Object prefix, Object oldValue, Object nextValue) {
        return oldValue;
    }

    static Object run() {
        Object value = new StringBuilder("old");
        return pick("prefix", (StringBuilder) value, value = new java.util.ArrayList<>());
    }

    public static void main(String[] args) {
        Object result = run();
        System.out.println(result.getClass().getName() + ":" + result.toString());
    }
}
