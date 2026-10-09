// jarde: presentation of `StringThenArray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StringThenArray extends java.lang.Object {
    public StringThenArray() {
        // @method <init>()V
        // @declaration a constructor of `StringThenArray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String run(java.lang.String arg0) {
        // @method run(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringThenArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local2 = arg0.trim();
        int local1 = local2.length();
        java.lang.String[] local2_2 = new java.lang.String[local1 + 1];
        local2_2[0] = arg0;
        local2_2[local1] = "!";
        return new java.lang.StringBuilder().append(local2_2[0]).append(local2_2[local1]).toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StringThenArray`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(" abc "));
        return;
    }
}
