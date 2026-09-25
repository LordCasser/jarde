import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;

final class PlatformIterableOwnersRunner {
    static List list(Object... values) {
        return Arrays.asList(values);
    }

    static Collection collection(Object... values) {
        return Arrays.asList(values);
    }

    static void failure(String label, Runnable action) {
        try {
            action.run();
            System.out.println(label + "=none");
        } catch (RuntimeException failure) {
            System.out.println(label + "=" + failure.getClass().getSimpleName());
        }
    }

    public static void main(String[] args) {
        PlatformIterableOwners.touches = 0;
        System.out.println("list=" + PlatformIterableOwners.listCast(list("a", "bc"))
                + ",touches=" + PlatformIterableOwners.touches);
        PlatformIterableOwners.touches = 0;
        System.out.println("collection=" + PlatformIterableOwners.collectionCast(
                collection("abc")) + ",touches=" + PlatformIterableOwners.touches);
        System.out.println("skip=" + PlatformIterableOwners.listSkipEmpty(list("", "zz")));
        PlatformIterableOwners.touches = 0;
        System.out.println("empty=" + PlatformIterableOwners.listCast(list())
                + ",touches=" + PlatformIterableOwners.touches);
        failure("listNull", new Runnable() {
            public void run() { PlatformIterableOwners.listCast(null); }
        });
        PlatformIterableOwners.touches = 0;
        failure("badCast", new Runnable() {
            public void run() { PlatformIterableOwners.listCast(list("ok", Integer.valueOf(7))); }
        });
        System.out.println("touchesAfterBadCast=" + PlatformIterableOwners.touches);
        System.out.println("twice=" + PlatformIterableOwners.listConsumesTwice(list("a", "bc")));
        System.out.println("escape=" + PlatformIterableOwners.collectionEscapes(collection("a")));
        TextIterable custom = new TextIterable() {
            public Iterator iterator() { return list("abcd").iterator(); }
        };
        IteratorSurface sameName = new IteratorSurface() {
            public Iterator iterator() { return list("abcde").iterator(); }
        };
        System.out.println("custom=" + PlatformIterableOwners.userSubtype(custom));
        System.out.println("sameName=" + PlatformIterableOwners.sameNameOnly(sameName));
    }
}
