

public class Y1 extends java.lang.Object {
    public Y1() {
        // @method <init>()V
        // @declaration a constructor of `Y1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String viaLambda(java.lang.String arg0) {
        // @method viaLambda(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Y1$StrFn local1 = (java.lang.String p0) -> Y1.lambda$viaLambda$0(p0);
        return local1.apply(arg0);
    }

    static int viaMethodRef() {
        // @method viaMethodRef()I
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.function.Supplier local0 = () -> Y1.lambda$viaMethodRef$1();
        java.util.function.Function local1 = (java.lang.Object p0) -> ((java.lang.String) p0).length();
        return ((java.lang.Integer) local0.get()).intValue() + ((java.lang.Integer) local1.apply((java.lang.Object) "hey")).intValue();
    }

    // jarde: generic Signature projection refused for `viaStream()Ljava/util/List;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.util.List viaStream() {
        // @method viaStream()Ljava/util/List;
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayList local0 = new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{"b", "aa", "ccc"}));
        local0.removeIf((java.util.function.Predicate) ((java.lang.Object p0) -> Y1.lambda$viaStream$2((java.lang.String) p0)));
        local0.sort((java.util.Comparator) ((java.lang.Object p0_, java.lang.Object p1) -> Y1.lambda$viaStream$3((java.lang.String) p0_, (java.lang.String) p1)));
        return local0;
    }

    static int captureLambda(int arg0) {
        // @method captureLambda(I)I
        // @declaration a static method of `Y1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.function.Function local1 = (java.lang.Object p0) -> Y1.lambda$captureLambda$4(arg0, (java.lang.Integer) p0);
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

    private static java.lang.Integer lambda$captureLambda$4(int arg0, java.lang.Integer arg1) {
        // @method lambda$captureLambda$4(ILjava/lang/Integer;)Ljava/lang/Integer;
        // @declaration a static method of `Y1`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(arg1.intValue() + arg0);
    }

    private static int lambda$viaStream$3(java.lang.String arg0, java.lang.String arg1) {
        // @method lambda$viaStream$3(Ljava/lang/String;Ljava/lang/String;)I
        // @declaration a static method of `Y1`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length() - arg1.length();
    }

    private static boolean lambda$viaStream$2(java.lang.String arg0) {
        // @method lambda$viaStream$2(Ljava/lang/String;)Z
        // @declaration a static method of `Y1`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.length() > 2;
    }

    private static java.lang.Integer lambda$viaMethodRef$1() {
        // @method lambda$viaMethodRef$1()Ljava/lang/Integer;
        // @declaration a static method of `Y1`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return 42;
    }

    private static java.lang.String lambda$viaLambda$0(java.lang.String arg0) {
        // @method lambda$viaLambda$0(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Y1`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + "!";
    }
}