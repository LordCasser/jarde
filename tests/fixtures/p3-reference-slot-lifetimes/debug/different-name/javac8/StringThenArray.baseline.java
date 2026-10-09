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

    static java.lang.String run(java.lang.String input) {
        // @method run(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringThenArray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String value = input.trim();
        int length = value.length();
        java.lang.String[] out = new java.lang.String[length + 1];
        out[0] = input;
        out[length] = "!";
        return new java.lang.StringBuilder().append(out[0]).append(out[length]).toString();
    }

    public static void main(java.lang.String[] x) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StringThenArray`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(" abc "));
        return;
    }
}
