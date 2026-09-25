package minimal;

import java.util.Objects;

/* JADX INFO: loaded from: UseObject.class */
public final class UseObject {
    private static int argumentCalls;

    private static int argument(int value) {
        argumentCalls++;
        return value;
    }

    public static Object make(Outer outer, int value) {
        Objects.requireNonNull(outer);
        return new Outer.Inner(outer, Integer.valueOf(argument(value)));
    }

    public static void main(String[] args) {
        Object result = make(new Outer(), 7);
        System.out.println(result.getClass().getName() + ":" + argumentCalls);
        try {
            make(null, 9);
        } catch (NullPointerException e) {
            System.out.println("null:" + argumentCalls);
        }
    }
}
