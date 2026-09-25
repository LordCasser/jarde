/// The two settlement shapes of P3 2.15, as **javac 23.0.1** compiles them for Java 8.
///
/// Every member states one reading of an exception edge, and the four differ in exactly the two
/// facts the walk settles an edge by:
///
/// * the record's range holds a **throwing instruction** or it does not — `javac --release 8`
///   protects the instructions a `try` body runs without asking whether any of them can raise, so
///   `try { n = n + 1; } catch (RuntimeException e)` is `iload_0; iconst_1; iadd; istore_0` under a
///   row nothing in the body can enter from;
/// * the range begins where its block begins or **inside** it — the canonical graph fuses
///   straight-line code, so a statement running before the `try` (`int x = 0;`) is the lead of the
///   same block the protected range starts in, and so is the binding store that opens a `catch`
///   clause whose body holds a `try` of its own (`nested`).
///
/// `increments` is the first shape with a named clause, `noThrowFinally` the same shape under the
/// catch-all row a `finally` copy is written under, `calls` the live half (`job.run()` is the
/// throwing instruction the record's range covers, and its range too begins inside its block), and
/// `nested` is the mid-block range of a statement written inside a clause.
public class Settled {
    static int increments(int n) {
        try {
            n = n + 1;
        } catch (RuntimeException e) {
            n = -1;
        }
        return n;
    }

    static int nested(int n) {
        try {
            n = n + 1;
        } catch (RuntimeException e) {
            try {
                n = -2;
            } catch (IllegalArgumentException e2) {
                n = -3;
            }
        }
        return n;
    }

    static int noThrowFinally(int n) {
        int x = 0;
        try {
            x = n;
        } finally {
            x = x + 1;
        }
        return x;
    }

    static int calls(int n, Runnable job) {
        int x = n;
        try {
            job.run();
        } catch (IllegalArgumentException e) {
            x = -4;
        }
        return x;
    }
}
