/// The behavior driver of the mid-read leg, compiled by the test with each leg's own compiler and
/// run against both the fixture's own class and the recovered text's recompiled class.
///
/// It exercises the two completions the `finally` covers:
///
/// * **normal** — three lines are counted and the close runs at the end;
/// * **mid-read** — the third read throws *inside* the protected range, the exception propagates
///   with its own message, and the close runs on that path too. `Failing.closed` records the
///   close, so a `finally` that did not run on the exceptional path — or one that ran twice — would
///   answer differently.
public final class IOMidReadDriver {
    private IOMidReadDriver() {}

    public static void main(String[] args) throws Exception {
        System.out.println(
                "normal="
                        + IOMidRead.countRemaining(
                                new java.io.BufferedReader(new java.io.StringReader("a\nbb\nccc\n"))));
        Failing failing = new Failing();
        try {
            System.out.println(
                    "unexpected="
                            + IOMidRead.countRemaining(new java.io.BufferedReader(failing)));
        } catch (java.io.IOException failure) {
            System.out.println("caught=" + failure.getMessage() + " closed=" + failing.closed);
        }
    }
}

/// A reader whose third read fails: two lines complete, the third throws, and `close` records that
/// it ran.
final class Failing extends java.io.Reader {
    private int reads;
    boolean closed;

    @Override
    public int read(char[] buffer, int offset, int length) throws java.io.IOException {
        reads++;
        if (reads == 3) {
            throw new java.io.IOException("read " + reads + " failed");
        }
        if (reads == 1 || reads == 2) {
            buffer[offset] = reads == 1 ? 'x' : 'y';
            buffer[offset + 1] = '\n';
            return 2;
        }
        return -1;
    }

    @Override
    public void close() {
        closed = true;
    }
}
