package defpackage;
import java.util.Arrays;

public final class NamedIteratorCursorRunner {
    public static void main(String[] args) {
        System.out.println("nonIterable=" + NamedIteratorCursor.sum(
                new NamedIteratorCursor(Arrays.asList("a", "bbb").iterator())));
    }
}
