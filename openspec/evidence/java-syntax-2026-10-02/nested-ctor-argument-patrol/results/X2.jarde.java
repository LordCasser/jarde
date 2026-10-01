// jarde: presentation of `X2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class X2 extends java.lang.Object {
    public X2() {
        // @method <init>()V
        // @declaration a constructor of `X2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String fmt(java.lang.Exception arg0) {
        // @method fmt(Ljava/lang/Exception;)Ljava/lang/String;
        // @declaration a static method of `X2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.getMessage();
    }

    public static java.lang.String single() {
        // @method single()Ljava/lang/String;
        // @declaration a static method of `X2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return fmt(new java.lang.Exception("solo"));
    }

    public static java.lang.String nested() {
        // jarde: not recovered: the recovery run for `nested()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nested()Ljava/lang/String;
        // @declaration a static method of `X2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21 18 15
        // the copy at BCI 3 has no proved local assignment
    }

    public static java.lang.String plainNested() {
        // @method plainNested()Ljava/lang/String;
        // @declaration a static method of `X2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf(new java.lang.Object());
    }

    public static java.lang.String nestedNew() {
        // @method nestedNew()Ljava/lang/String;
        // @declaration a static method of `X2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf((java.lang.Object) new java.lang.StringBuilder("sb"));
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `X2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) single());
        java.lang.System.out.println((java.lang.String) nested());
        java.lang.System.out.println((java.lang.String) nestedNew());
        return;
    }
}
