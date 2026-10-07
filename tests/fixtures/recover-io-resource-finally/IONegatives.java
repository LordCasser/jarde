/// The two shapes the certificate must **not** claim, each one link away from the anchor. Both are
/// verifier-valid Java 8 classes compiled from this source, so a refusal is evidence about the
/// proof rather than about damaged bytes.
///
/// * `twoNested()` — two resources with two nested `finally` clauses: the table states **two**
///   handlers, so the row set does not cover one `finally`, and the certificate — whose rows all
///   reach the same handler — keeps the whole method refused.
/// * `closeReturns()` — a cleanup call that returns a value: the copies are
///   `aload; invoke close()I; pop`, not the value-less two-instruction `close()` the certificate
///   proves, so the shape keeps its refusal.
public final class IONegatives {
    private IONegatives() {}

    static int twoNested(String first, String second) throws java.io.IOException {
        java.io.FileReader outer = new java.io.FileReader(first);
        java.io.FileReader inner = new java.io.FileReader(second);
        try {
            try {
                int n = 0;
                while (inner.read() != -1) {
                    n++;
                }
                return n;
            } finally {
                inner.close();
            }
        } finally {
            outer.close();
        }
    }

    static int closeReturns(String path) throws java.io.IOException {
        Returner r = new Returner(path);
        try {
            int n = 0;
            while (r.read() != -1) {
                n++;
            }
            return n;
        } finally {
            r.close();
        }
    }
}

/// The handle whose `close` answers a value, so its copies carry the discard the certificate does
/// not state.
final class Returner {
    Returner(String path) {}

    int read() {
        return -1;
    }

    int close() {
        return 0;
    }
}
