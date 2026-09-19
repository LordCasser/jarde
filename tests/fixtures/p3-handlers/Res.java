/// The resource the guarded statements of this fixture close.
///
/// It prints what it is asked to do, so the *order* of the opens and the closes is observable from a
/// plain `java` run — which is what the verification record compares between the original bodies and
/// the presented ones. `close` can be made to fail, so that a body which throws and a resource which
/// throws while closing are both reachable: the primary/suppressed relationship of the exceptional
/// path is only observable at run time.
public class Res implements AutoCloseable {
    private final String tag;
    private final boolean fails;

    public Res(String tag, boolean fails) {
        this.tag = tag;
        this.fails = fails;
    }

    public void close() {
        System.out.println("close " + tag);
        if (fails) {
            throw new IllegalStateException("close-" + tag);
        }
    }
}
