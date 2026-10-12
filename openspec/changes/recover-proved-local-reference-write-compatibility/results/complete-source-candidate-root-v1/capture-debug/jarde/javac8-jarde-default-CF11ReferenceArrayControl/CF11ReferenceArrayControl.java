// jarde: presentation of `cf11/CF11ReferenceArrayControl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11ReferenceArrayControl extends java.lang.Object {
    public CF11ReferenceArrayControl() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11ReferenceArrayControl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int lengthAt(boolean useInterfaceArray) {
        // @method lengthAt(Z)I
        // @declaration a static method of `cf11.CF11ReferenceArrayControl`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String[] target;
        target = new java.lang.String[1];
        if (useInterfaceArray) {
            target = new java.lang.CharSequence[1];
        }
        target[0] = "x";
        return target[0].length();
    }
}
