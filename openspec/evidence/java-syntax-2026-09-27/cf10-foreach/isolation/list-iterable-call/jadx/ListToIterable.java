package defpackage;

import java.util.Arrays;

/* JADX INFO: loaded from: ListToIterable.class */
public class ListToIterable {
    public static void consume(Iterable values) {
        System.out.println("called");
    }

    public static void main(String[] args) {
        consume(Arrays.asList("a", "b", "c"));
    }
}
