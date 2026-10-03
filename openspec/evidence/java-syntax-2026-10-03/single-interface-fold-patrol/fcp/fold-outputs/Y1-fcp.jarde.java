// jarde: presentation of `Y1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Y1 extends java.lang.Object {
    // jarde: omitted physical lambda helper "lambda$viaLambda$0" after proving its single class-wide use
    // jarde: omitted physical lambda helper "lambda$viaMethodRef$1" after proving its single class-wide use
    // jarde: omitted physical lambda helper "lambda$viaStream$2" after proving its single class-wide use
    // jarde: omitted physical lambda helper "lambda$viaStream$3" after proving its single class-wide use
    // jarde: omitted physical lambda helper "lambda$captureLambda$4" after proving its single class-wide use
    public Y1() {
        // @method <init>()V
        // @declaration a constructor of `Y1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String viaLambda(java.lang.String arg0) {
        // jarde: lambda companion body inlined at invokedynamic@0
        // @method viaLambda(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        StrFn local1 = (java.lang.String p0) -> p0 + "!";
        return local1.apply(arg0);
    }

    static int viaMethodRef() {
        // jarde: lambda companion body inlined at invokedynamic@0
        // @method viaMethodRef()I
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.function.Supplier local0 = () -> 42;
        java.util.function.Function local1 = (java.lang.Object p0) -> ((java.lang.String) p0).length();
        return ((java.lang.Integer) local0.get()).intValue() + ((java.lang.Integer) local1.apply((java.lang.Object) "hey")).intValue();
    }

    static java.util.List viaStream() {
        // jarde: generic Signature projection refused for `viaStream()Ljava/util/List;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
        // jarde: lambda companion body inlined at invokedynamic@31
        // jarde: lambda companion body inlined at invokedynamic@43
        // @method viaStream()Ljava/util/List;
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayList local0 = new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{"b", "aa", "ccc"}));
        local0.removeIf((java.util.function.Predicate) ((java.lang.Object arg0) -> ((java.lang.String) arg0).length() > 2));
        local0.sort((java.util.Comparator) ((java.lang.Object arg0, java.lang.Object arg1) -> ((java.lang.String) arg0).length() - ((java.lang.String) arg1).length()));
        return local0;
    }

    static int captureLambda(int arg0) {
        // jarde: lambda companion body inlined at invokedynamic@1
        // @method captureLambda(I)I
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.function.Function local1 = (java.lang.Object arg1) -> java.lang.Integer.valueOf(((java.lang.Integer) arg1).intValue() + arg0);
        return ((java.lang.Integer) local1.apply((java.lang.Object) java.lang.Integer.valueOf(5))).intValue();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Y1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) viaLambda("hi"));
        java.lang.System.out.println(viaMethodRef());
        java.lang.System.out.println((java.lang.Object) viaStream());
        java.lang.System.out.println(captureLambda(3));
        return;
    }

    static interface StrFn {
        // jarde: no body: the member `apply(Ljava/lang/String;)Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
        public abstract java.lang.String apply(java.lang.String arg1);
    }
}
