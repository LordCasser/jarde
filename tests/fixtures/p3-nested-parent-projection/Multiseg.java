/// The multi-segment boundary form: the class's own `Signature` states the superclass as the
/// binary path `LMO<Ljava/lang/String;>.Mid;` — a path whose middle segment carries the type
/// arguments and whose own name is joined with `$` (`MO$Mid`). The one-segment direct-parent
/// candidate never covers it, so the header stays refused exactly as before.
public class Multiseg extends MO<java.lang.String>.Mid {
    public Multiseg(MO<java.lang.String> outer) {
        outer.super();
    }

    public static void main(String[] args) {
        new Multiseg(new MO<java.lang.String>());
    }
}
