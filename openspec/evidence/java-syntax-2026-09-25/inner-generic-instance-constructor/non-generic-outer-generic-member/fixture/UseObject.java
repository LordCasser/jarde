package minimal;

public final class UseObject {
    private static int argumentCalls;

    private static int argument(int value) {
        argumentCalls++;
        return value;
    }

    public static Object make(Outer outer, int value) {
        return outer.new Inner<>(argument(value));
    }

    public static void main(String[] args) {
        Object result = make(new Outer(), 7);
        System.out.println(result.getClass().getName() + ":" + argumentCalls);
        try {
            make(null, 9);
        } catch (NullPointerException expected) {
            System.out.println("null:" + argumentCalls);
        }
    }
}
