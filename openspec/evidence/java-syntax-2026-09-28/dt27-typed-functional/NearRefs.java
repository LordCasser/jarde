package dt27;

import java.util.function.Function;

public class NearRefs {
    private NearRefs holder;
    private final int offset;
    private int calls;

    public NearRefs(int offset) { this.offset = offset; }
    private Integer length(String text) { return text.length() + offset; }
    private NearRefs next() { calls++; return this; }

    public Function<String, Integer> effect() {
        calls++;
        return this::length;
    }

    public Function<String, Integer> handled() {
        try { return this::length; }
        catch (RuntimeException failure) { return Integer::parseInt; }
    }

    public Function<String, Integer> nullable() { return holder::length; }
    public Function<String, Integer> evaluated() { return next()::length; }

    public Function<String, Integer> replaced() {
        NearRefs old = holder;
        Function<String, Integer> function = old::length;
        holder = new NearRefs(100);
        return function;
    }

    public static void main(String[] args) {
        NearRefs root = new NearRefs(2);
        System.out.println("effect=" + root.effect().apply("abc") + ":calls=" + root.calls);
        System.out.println("handled=" + root.handled().apply("abc"));
        try { root.nullable(); System.out.println("nullable=missing-npe"); }
        catch (NullPointerException expected) { System.out.println("nullable=creation-npe"); }
        System.out.println("evaluated=" + root.evaluated().apply("abc") + ":calls=" + root.calls);
        root.holder = new NearRefs(4);
        Function<String, Integer> saved = root.replaced();
        System.out.println("replaced=" + saved.apply("abc") + ":holder=" + root.holder.offset);
    }
}
