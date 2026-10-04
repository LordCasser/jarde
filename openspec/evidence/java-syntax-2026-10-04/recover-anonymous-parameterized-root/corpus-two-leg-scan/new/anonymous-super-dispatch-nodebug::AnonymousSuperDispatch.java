// jarde: presentation of `AnonymousSuperDispatch` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class AnonymousSuperDispatch extends java.lang.Object {
    static boolean inBaseConstructor;

    static boolean capturedVisibleBeforeBaseReturns;

    static java.lang.String observed;

    public AnonymousSuperDispatch() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousSuperDispatch`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static Base create(java.lang.String arg0) {
        return new Base() {
            void observe() {
                AnonymousSuperDispatch.observed = arg0;
                return;
            }
        };
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousSuperDispatch`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        create("captured-value");
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("observed=").append(AnonymousSuperDispatch.observed).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("visibleDuringSuper=").append(AnonymousSuperDispatch.capturedVisibleBeforeBaseReturns).toString());
        return;
    }
}
