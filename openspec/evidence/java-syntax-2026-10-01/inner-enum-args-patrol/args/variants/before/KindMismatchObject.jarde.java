// jarde: presentation of `KindMismatchObject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LKindMismatchObject;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
enum KindMismatchObject {
    public static final KindMismatchObject A;

    private final java.lang.Object v;

    private static final KindMismatchObject[] $VALUES;

    public static KindMismatchObject[] values() {
        // @method values()[LKindMismatchObject;
        // @declaration a static method of `KindMismatchObject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (KindMismatchObject[]) KindMismatchObject.$VALUES.clone();
    }

    public static KindMismatchObject valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)LKindMismatchObject;
        // @declaration a static method of `KindMismatchObject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (KindMismatchObject) java.lang.Enum.valueOf(KindMismatchObject.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;ILjava/lang/Object;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private KindMismatchObject(java.lang.String arg1, int arg2, java.lang.Object arg3) {
        // @method <init>(Ljava/lang/String;ILjava/lang/Object;)V
        // @declaration a constructor of `KindMismatchObject`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.v = arg3;
        return;
    }

    java.lang.Object v() {
        // @method v()Ljava/lang/Object;
        // @declaration an instance method of `KindMismatchObject`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.v;
    }

    private static KindMismatchObject[] $values() {
        // @method $values()[LKindMismatchObject;
        // @declaration a static method of `KindMismatchObject`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new KindMismatchObject[]{KindMismatchObject.A};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `KindMismatchObject`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        KindMismatchObject.A = new KindMismatchObject("A", 0, (java.lang.Object) "plain");
        KindMismatchObject.$VALUES = $values();
    }
}
