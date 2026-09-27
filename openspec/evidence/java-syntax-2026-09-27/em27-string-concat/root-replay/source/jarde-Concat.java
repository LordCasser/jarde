// jarde: presentation of `em27/Concat` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em27;

public final class Concat extends java.lang.Object {
    private Concat() {
        // @method <init>()V
        // @declaration a constructor of `em27.Concat`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String builder(int arg0) {
        // @method builder(I)Ljava/lang/String;
        // @declaration a static method of `em27.Concat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "Value" + " equals " + arg0;
    }

    public static java.lang.String folded() {
        // @method folded()Ljava/lang/String;
        // @declaration a static method of `em27.Concat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "App " + "version: " + 1 + '.' + 2;
    }

    public static java.lang.String objects(java.lang.Object arg0, java.lang.Object arg1) {
        // @method objects(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `em27.Concat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local2 = new java.lang.StringBuilder();
        local2.append(arg0);
        local2.append('=');
        local2.append(arg1);
        return local2.toString();
    }

    public static java.lang.String character(java.lang.String arg0) {
        // @method character(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `em27.Concat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "" + '1' + arg0 + ", e: " + 2;
    }

    public static void discarded(int arg0) {
        // @method discarded(I)V
        // @declaration a static method of `em27.Concat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1 = "Input arg value: " + arg0;
        return;
    }

    public static java.lang.String explicitConstructor() {
        // @method explicitConstructor()Ljava/lang/String;
        // @declaration a static method of `em27.Concat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.String(new char[]{'a', 'b', 'c'});
    }

    public static java.lang.String explicitStored() {
        // @method explicitStored()Ljava/lang/String;
        // @declaration a static method of `em27.Concat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char[] local0 = new char[]{'a', 'b', 'c'};
        return new java.lang.String(local0);
    }
}
