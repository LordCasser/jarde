package flow;

public class Consumer {
    static int select(Object x) { return 1; }
    static int select(java.util.List<identity.Foo> x) { return 2; }

    public static int rawFieldOverload(Base<java.util.List<identity.Foo>> input) {
        Base raw = input;
        return select(raw.baseValue);
    }

    public static void rawAdd(java.util.List<identity.Foo> input, Object value) {
        java.util.List raw = input;
        raw.add((Object) value);
    }

    public static boolean rawListEquality(
            java.util.List<identity.Foo> a, java.util.List<String> b) {
        java.util.List rawA = a;
        java.util.List rawB = b;
        return rawA == rawB;
    }
}
