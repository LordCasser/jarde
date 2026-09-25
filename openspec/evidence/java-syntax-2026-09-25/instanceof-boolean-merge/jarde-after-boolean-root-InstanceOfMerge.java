// jarde: presentation of `InstanceOfMerge` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class InstanceOfMerge extends java.lang.Object {
    static int calls;

    public InstanceOfMerge() {
        // @method <init>()V
        // @declaration a constructor of `InstanceOfMerge`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.Object value(java.lang.Object arg0) {
        // @method value(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `InstanceOfMerge`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        InstanceOfMerge.calls = InstanceOfMerge.calls + 1;
        return arg0;
    }

    static boolean inverted(java.lang.Object arg0) {
        // @method inverted(Ljava/lang/Object;)Z
        // @declaration a static method of `InstanceOfMerge`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return !(value(arg0) instanceof java.lang.String);
    }

    static boolean direct(java.lang.Object arg0) {
        // @method direct(Ljava/lang/Object;)Z
        // @declaration a static method of `InstanceOfMerge`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return value(arg0) instanceof java.lang.String;
    }
}
