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

    static java.lang.String run(java.lang.String arg0) {
        // @method run(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `BuilderThenArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        char[] local2_2;
        int local3;
        java.lang.StringBuilder local2 = new java.lang.StringBuilder(arg0);
        local1 = local2.length();
        local2_2 = new char[local1];
        for (local3 = 0; local3 < local1; local3 = local3 + 1) {
            local2_2[local3] = arg0.charAt(local3);
        }
        return new java.lang.String(local2_2);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BuilderThenArray`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run("abc"));
        return;
    }
}
