// jarde: presentation of `cf11/CF11CharSequenceControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11CharSequenceControl extends java.lang.Object {
    public CF11CharSequenceControl() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11CharSequenceControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int stringLength() {
        // @method stringLength()I
        // @declaration a static method of `cf11.CF11CharSequenceControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local0 = "string";
        return local0.length();
    }

    public static int builderLength() {
        // @method builderLength()I
        // @declaration a static method of `cf11.CF11CharSequenceControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local0 = "seed";
        java.lang.StringBuilder local0_2 = new java.lang.StringBuilder("builder");
        return local0_2.length();
    }
}
