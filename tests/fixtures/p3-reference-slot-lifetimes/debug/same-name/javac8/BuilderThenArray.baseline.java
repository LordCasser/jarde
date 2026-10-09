// jarde: presentation of `BuilderThenArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BuilderThenArray extends java.lang.Object {
    public BuilderThenArray() {
        // @method <init>()V
        // @declaration a constructor of `BuilderThenArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String run(java.lang.String x) {
        // @method run(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `BuilderThenArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int n;
        java.lang.StringBuilder v;
        int i;
        v = new java.lang.StringBuilder(x);
        n = v.length();
        v = new char[n];
        for (i = 0; i < n; i = i + 1) {
            v[i] = x.charAt(i);
        }
        // @bytecode 47
        // the parameter 0 of the invocation at BCI 44 is declared `char[]` presents `java.lang.StringBuilder` but the invocation requires `char[]` and this layer has no safe reference conversion evidence
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BuilderThenArray`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run("abc"));
        return;
    }
}
