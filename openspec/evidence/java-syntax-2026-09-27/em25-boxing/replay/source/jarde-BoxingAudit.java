// jarde: presentation of `em25/BoxingAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em25;

public final class BoxingAudit extends java.lang.Object {
    private BoxingAudit() {
        // @method <init>()V
        // @declaration a constructor of `em25.BoxingAudit`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object boxInteger() {
        // @method boxInteger()Ljava/lang/Object;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(1);
    }

    public static java.lang.Object boxBoolean() {
        // @method boxBoolean()Ljava/lang/Object;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Boolean.valueOf(true);
    }

    public static java.lang.Object boxByte() {
        // @method boxByte()Ljava/lang/Object;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Byte.valueOf((byte) 2);
    }

    public static java.lang.Short boxShort() {
        // @method boxShort()Ljava/lang/Short;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Short.valueOf((short) 3);
    }

    public static java.lang.Character boxCharacter() {
        // @method boxCharacter()Ljava/lang/Character;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Character.valueOf('c');
    }

    public static java.lang.Long boxLong() {
        // @method boxLong()Ljava/lang/Long;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Long.valueOf(4L);
    }

    public static long unboxOrDefault(java.lang.Long arg0) {
        // @method unboxOrDefault(Ljava/lang/Long;)J
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == null) {
            arg0 = java.lang.Long.valueOf(0L);
        }
        return arg0.longValue();
    }

    public static boolean unbox(java.lang.Boolean arg0) {
        // @method unbox(Ljava/lang/Boolean;)Z
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.booleanValue();
    }
}
