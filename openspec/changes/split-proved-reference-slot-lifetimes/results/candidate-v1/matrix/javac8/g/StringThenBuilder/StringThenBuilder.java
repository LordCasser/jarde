// jarde: presentation of `StringThenBuilder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StringThenBuilder extends java.lang.Object {
    public StringThenBuilder() {
        // @method <init>()V
        // @declaration a constructor of `StringThenBuilder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String run(java.lang.String input) {
        // @method run(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringThenBuilder`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String value = input.trim();
        int length = value.length();
        java.lang.StringBuilder out = new java.lang.StringBuilder();
        out.append(length).append(':').append(input.length());
        return out.toString();
    }

    public static void main(java.lang.String[] x) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StringThenBuilder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(" abc "));
        return;
    }
}
