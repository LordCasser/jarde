package minimal;

import java.util.Objects;

/* JADX INFO: loaded from: UseObject.class */
public final class UseObject {
    private static int argumentCalls;

    private static int argument(int i) {
        argumentCalls++;
        return i;
    }

    public static Object make(Outer outer, int i) {
        Objects.requireNonNull(outer);
        return new Outer.Inner(outer, Integer.valueOf(argument(i)));
    }

    public static void main(String[] strArr) {
        System.out.println(make(new Outer(), 7).getClass().getName() + ":" + argumentCalls);
        try {
            make(null, 9);
        } catch (NullPointerException e) {
            System.out.println("null:" + argumentCalls);
        }
    }
}
