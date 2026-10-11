// jarde: presentation of `cf11/CF11SlotReuseControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11SlotReuseControl extends java.lang.Object {
    public CF11SlotReuseControl() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11SlotReuseControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object afterStringLifetime(boolean replace, java.util.List values) {
        // @method afterStringLifetime(ZLjava/util/List;)Ljava/lang/Object;
        // @declaration a static method of `cf11.CF11SlotReuseControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local2;
        local2 = "a";
        local2.length();
        local2 = "seed";
        if (replace) {
            local2 = values.get(0);
        }
        return local2;
    }
}
