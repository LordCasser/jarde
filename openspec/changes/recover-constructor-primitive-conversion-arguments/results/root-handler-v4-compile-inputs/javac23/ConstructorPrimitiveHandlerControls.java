// jarde: presentation of `ConstructorPrimitiveHandlerControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ConstructorPrimitiveHandlerControls extends java.lang.Object {
    public ConstructorPrimitiveHandlerControls() {
        // @method <init>()V
        // @declaration a constructor of `ConstructorPrimitiveHandlerControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int markInt(int arg0) {
        // @method markInt(I)I
        // @declaration a static method of `ConstructorPrimitiveHandlerControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print("handler:");
        java.lang.System.out.println(arg0);
        return arg0;
    }

    static java.lang.Long identity(java.lang.Long arg0) {
        // @method identity(Ljava/lang/Long;)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveHandlerControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    static java.lang.Long sameHandler(int arg0) {
        // @method sameHandler(I)Ljava/lang/Long;
        // @declaration a static method of `ConstructorPrimitiveHandlerControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            return identity(new java.lang.Long((long) markInt(arg0)));
        } catch (java.lang.RuntimeException local1) {
            return null;
        }
    }
}
