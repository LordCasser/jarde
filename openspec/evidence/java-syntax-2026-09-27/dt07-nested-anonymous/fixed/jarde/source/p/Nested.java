// jarde: presentation of `p/Nested` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

public final class Nested extends java.lang.Object {
    public static int trace;

    public Nested() {
        // @method <init>()V
        // @declaration a constructor of `p.Nested`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static p.Factory create() {
        return new p.Factory() {
            public p.Action make() {
                return new p.Action() {
                    public void run() {
                        p.Nested.trace = p.Nested.trace + 1;
                        return;
                    }
                };
            }
        };
    }
}
