// jarde: presentation of `SB` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class SB extends java.lang.Object {
    public SB() {
        // @method <init>()V
        // @declaration a constructor of `SB`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `acc(Ljava/util/List;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    static java.lang.String acc(java.util.List arg0) {
        // @method acc(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `SB`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        local1 = new java.lang.StringBuilder();
        for (java.lang.Object iteratorElement25 : arg0) {
            java.lang.String local3 = (java.lang.String) iteratorElement25;
            local1.append(local3).append(',');
        }
        return local1.toString();
    }

    // jarde: generic Signature projection refused for `cond(Ljava/util/List;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    static java.lang.String cond(java.util.List arg0) {
        // @method cond(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `SB`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        java.util.Iterator local2;
        local1 = "";
        local2 = arg0.iterator();
        while (local2.hasNext()) {
            java.lang.String local3 = (java.lang.String) local2.next();
            if (local3.length() > 1) {
                local1 = local1 + local3;
            }
        }
        return local1;
    }

    static int[] copy(int[] arg0) {
        // @method copy([I)[I
        // @declaration a static method of `SB`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1 = new int[arg0.length];
        java.lang.System.arraycopy((java.lang.Object) arg0, 0, (java.lang.Object) local1, 0, arg0.length);
        return local1;
    }

    static int[] of(int[] arg0) {
        // @method of([I)[I
        // @declaration a static method of `SB`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.util.Arrays.copyOf(arg0, arg0.length + 1);
    }

    static java.lang.String join(java.lang.String[] arg0) {
        // @method join([Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `SB`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.join((java.lang.CharSequence) "-", (java.lang.CharSequence[]) arg0);
    }

    static int spread(int... arg0) {
        // @method spread([I)I
        // @declaration a static method of `SB`, member flags 0x0088
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int[] local2;
        local1 = 0;
        local2 = arg0;
        for (int local5 : local2) {
            local1 = local1 + local5;
        }
        return local1;
    }

    static int call() {
        // @method call()I
        // @declaration a static method of `SB`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return spread(1, 2, 3);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `SB`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.io.PrintStream saved0 = java.lang.System.out;
        java.lang.StringBuilder saved1 = new java.lang.StringBuilder();
        java.lang.StringBuilder saved2 = saved1.append((java.lang.String) acc((java.util.List) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{"a", "b"}))).append("/");
        java.lang.StringBuilder saved3 = saved2.append((java.lang.String) cond((java.util.List) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{"x", "yy"}))).append("/");
        java.lang.StringBuilder saved4 = saved3.append((java.lang.String) java.util.Arrays.toString((int[]) copy(new int[]{1, 2}))).append("/");
        java.lang.StringBuilder saved5 = saved4.append((java.lang.String) java.util.Arrays.toString((int[]) of(new int[]{1}))).append("/");
        java.lang.StringBuilder saved6 = saved5.append((java.lang.String) join(new java.lang.String[]{"p", "q"})).append("/");
        saved0.println((java.lang.String) saved6.append(spread(4, 5)).append("/").append(call()).toString());
        return;
    }
}
