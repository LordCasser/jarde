// jarde: presentation of `Main` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Main extends java.lang.Object {
    public Main() {
        // @method <init>()V
        // @declaration a constructor of `Main`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void event(java.lang.String arg0, java.lang.String arg1) {
        // @method event(Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print(arg0);
        java.lang.System.out.print(':');
        java.lang.System.out.println(arg1);
        return;
    }

    public static java.lang.String mark(java.lang.String arg0) {
        // @method mark(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        event("mark", arg0);
        return arg0;
    }

    public static java.lang.CharSequence[] sequence() {
        // @method sequence()[Ljava/lang/CharSequence;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.CharSequence[]{new java.lang.StringBuilder((java.lang.String) mark("sequence-first")), new java.lang.StringBuffer((java.lang.String) mark("sequence-second"))};
    }

    // jarde: generic Signature projection refused for `collections()[Ljava/util/Collection;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public static java.util.Collection[] collections() {
        // @method collections()[Ljava/util/Collection;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.Collection[]{new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{mark("collection-first")})), new java.util.HashSet((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{mark("collection-second")}))};
    }

    public static java.lang.Throwable[] failures() {
        // @method failures()[Ljava/lang/Throwable;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Throwable[]{new java.lang.IllegalStateException((java.lang.String) mark("throwable-first")), new java.lang.IllegalArgumentException((java.lang.String) mark("throwable-second"))};
    }

    public static Base[] ownDirect() {
        // @method ownDirect()[LBase;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new Base[]{new DirectA((java.lang.String) mark("direct-first")), new DirectB((java.lang.String) mark("direct-second"))};
    }

    public static Base[] ownTwoHop() {
        // @method ownTwoHop()[LBase;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new Base[]{new TwoHop((java.lang.String) mark("two-hop-first")), new DirectB((java.lang.String) mark("two-hop-second"))};
    }

    public static LocalInterface[] ownInterface() {
        // @method ownInterface()[LLocalInterface;
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new LocalInterface[]{new DirectA((java.lang.String) mark("interface-first")), new TwoHop((java.lang.String) mark("interface-second"))};
    }

    private static void observe(java.lang.Object[] arg0) {
        // @method observe([Ljava/lang/Object;)V
        // @declaration a static method of `Main`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object local1 = arg0[0];
        if (local1 == null) {
            java.lang.System.out.println("null");
        } else {
            java.lang.System.out.println((java.lang.String) local1.getClass().getName());
        }
        java.lang.Object local2 = arg0[1];
        if (local2 == null) {
            java.lang.System.out.println("null");
        } else {
            java.lang.System.out.println((java.lang.String) local2.getClass().getName());
        }
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        observe((java.lang.Object[]) sequence());
        observe((java.lang.Object[]) collections());
        observe((java.lang.Object[]) failures());
        observe((java.lang.Object[]) ownDirect());
        observe((java.lang.Object[]) ownTwoHop());
        observe((java.lang.Object[]) ownInterface());
        return;
    }
}
