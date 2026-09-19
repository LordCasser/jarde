/// The guarded statements of P3 2.4, as **javac 23.0.1** compiles them for Java 8.
///
/// Every member is a shape the recovery layer is asked about, and the shapes come in two kinds:
///
/// * the ones the `twr@1`/`monitor@1` rules prove — a `try`‑with‑resources with one, two and three
///   resources, one whose second resource's own initialisation throws, one whose body throws, a
///   `synchronized` block;
/// * the ones they must refuse — a `try`‑with‑resources with a `catch` beside it (the compiler wraps
///   the whole construct in a row of its own), a guarded body that branches, and the two `finally`
///   shapes.
///
/// `open`/`openFailing`/`fail` print, and so does `body`/`tail`, so a run of `main` states the
/// order of every open and every close, and the suppressed relationship of the exceptional path.
public class Guarded {
    static final Object LOCK = new Object();
    static boolean FLAG = true;

    static Res open(String tag) {
        System.out.println("open " + tag);
        return new Res(tag, false);
    }

    static Res openFailing(String tag) {
        System.out.println("open " + tag);
        return new Res(tag, true);
    }

    static Res fail(String tag) {
        System.out.println("fail " + tag);
        throw new IllegalStateException("open-" + tag);
    }

    static void body() {
        System.out.println("body");
    }

    static void tail() {
        System.out.println("tail");
    }

    static void boom() {
        throw new IllegalStateException("boom");
    }

    static void one() {
        try (Res r = open("r")) {
            body();
        }
    }

    static void two() {
        try (Res r = open("r"); Res s = open("s")) {
            body();
        }
    }

    static void three() {
        try (Res r = open("r"); Res s = open("s"); Res t = open("t")) {
            body();
        }
    }

    static void suppressed() {
        try (Res r = openFailing("r")) {
            boom();
        }
    }

    static void secondInitFails() {
        try (Res r = open("r"); Res s = fail("s")) {
            body();
        }
    }

    static void sync() {
        synchronized (LOCK) {
            body();
        }
    }

    static void syncBody() {
        synchronized (Guarded.class) {
            body();
            tail();
        }
    }

    static void syncThrows() {
        synchronized (LOCK) {
            boom();
        }
    }

    static void withCatch() {
        try (Res r = open("r")) {
            body();
        } catch (RuntimeException e) {
            tail();
        }
    }

    static void branching() {
        try (Res r = open("r")) {
            if (FLAG) {
                body();
            }
        }
    }

    static void fin() {
        try {
            body();
        } finally {
            tail();
        }
    }

    static void catchFinally() {
        try {
            boom();
        } catch (RuntimeException e) {
            tail();
        } finally {
            body();
        }
    }

    public static void main(String[] args) {
        one();
        two();
        three();
        sync();
        syncBody();
        syncThrowsCatching();
        secondInitFailsCatching();
        suppressedCatching();
        branching();
        withCatch();
        fin();
        catchFinally();
    }

    static void syncThrowsCatching() {
        try {
            syncThrows();
        } catch (RuntimeException e) {
            System.out.println("caught " + e.getMessage());
        }
    }

    static void secondInitFailsCatching() {
        try {
            secondInitFails();
        } catch (RuntimeException e) {
            System.out.println("caught " + e.getMessage());
        }
    }

    static void suppressedCatching() {
        try {
            suppressed();
        } catch (RuntimeException e) {
            System.out.println("caught " + e.getMessage());
            for (Throwable suppressed : e.getSuppressed()) {
                System.out.println("suppressed " + suppressed.getMessage());
            }
        }
    }
}
