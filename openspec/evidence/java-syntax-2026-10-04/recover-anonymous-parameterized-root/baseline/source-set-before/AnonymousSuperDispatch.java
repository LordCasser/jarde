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

    private static Base create(java.lang.String captured) {
        // @method create(Ljava/lang/String;)LBase;
        // @declaration a static method of `AnonymousSuperDispatch`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new AnonymousSuperDispatch$1(captured);
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousSuperDispatch`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        create("captured-value");
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("observed=").append(AnonymousSuperDispatch.observed).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("visibleDuringSuper=").append(AnonymousSuperDispatch.capturedVisibleBeforeBaseReturns).toString());
        return;
    }
}
