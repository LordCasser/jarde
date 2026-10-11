// jarde: presentation of `cf11/CF11CastReadControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11CastReadControl extends java.lang.Object {
    public CF11CastReadControl() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11CastReadControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int castRead(java.util.List values) {
        // @method castRead(Ljava/util/List;)I
        // @declaration a static method of `cf11.CF11CastReadControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String target;
        target = "seed";
        if (!values.isEmpty()) {
            target = values.get(0);
        }
        return ((java.lang.String) target).length();
    }
}
