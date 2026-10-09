// jarde: presentation of `CT` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CT extends java.lang.Object {
    public CT() {
        // @method <init>()V
        // @declaration a constructor of `CT`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String io(java.lang.Object arg0) {
        // @method io(Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `CT`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 instanceof java.lang.String) {
            java.lang.String local1 = (java.lang.String) arg0;
            return local1.toUpperCase();
        } else {
            return "?";
        }
    }

    static java.lang.String io2(java.lang.Object arg0) {
        // @method io2(Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `CT`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 instanceof java.lang.String ? ((java.lang.String) arg0).trim() : "?";
    }

    static java.lang.Object up() {
        // @method up()Ljava/lang/Object;
        // @declaration a static method of `CT`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.List local0 = java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{"a", "b"});
        return local0;
    }

    static java.lang.Number cov() {
        // @method cov()Ljava/lang/Number;
        // @declaration a static method of `CT`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.List local0 = java.util.Arrays.asList((java.lang.Object[]) new java.lang.Number[]{java.lang.Integer.valueOf(1), java.lang.Long.valueOf(2L)});
        return (java.lang.Number) local0.get(0);
    }

    static int prim() {
        // @method prim()I
        // @declaration a static method of `CT`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Double local0 = java.lang.Double.valueOf((double) java.lang.Integer.valueOf(3).intValue());
        return ((java.lang.Number) local0).intValue();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `CT`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(io((java.lang.Object) "x") + "/" + io2((java.lang.Object) " y ") + "/" + up() + "/" + cov() + "/" + prim());
        return;
    }
}
