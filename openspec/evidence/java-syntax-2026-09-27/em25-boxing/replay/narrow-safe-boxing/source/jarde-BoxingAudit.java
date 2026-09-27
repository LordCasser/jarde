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
        return 1;
    }

    public static java.lang.Integer integerMinimum() {
        // @method integerMinimum()Ljava/lang/Integer;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return -128;
    }

    public static java.lang.Integer integerMaximum() {
        // @method integerMaximum()Ljava/lang/Integer;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 127;
    }

    public static java.lang.Integer integerBelowRange() {
        // @method integerBelowRange()Ljava/lang/Integer;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(-129);
    }

    public static java.lang.Integer integerAboveRange() {
        // @method integerAboveRange()Ljava/lang/Integer;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(128);
    }

    public static java.lang.Number integerAsNumber() {
        // @method integerAsNumber()Ljava/lang/Number;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(1);
    }

    public static java.lang.Integer integerConsumedByCall() {
        // @method integerConsumedByCall()Ljava/lang/Integer;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return retain((java.lang.Integer) java.lang.Integer.valueOf(1));
    }

    private static java.lang.Integer retain(java.lang.Integer arg0) {
        // @method retain(Ljava/lang/Integer;)Ljava/lang/Integer;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static java.lang.Object boxBoolean() {
        // @method boxBoolean()Ljava/lang/Object;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return true;
    }

    public static java.lang.Object boxFalse() {
        // @method boxFalse()Ljava/lang/Object;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return false;
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
        return 'c';
    }

    public static java.lang.Character characterAsciiMaximum() {
        // @method characterAsciiMaximum()Ljava/lang/Character;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return '\u007f';
    }

    public static java.lang.Character characterAboveAscii() {
        // @method characterAboveAscii()Ljava/lang/Character;
        // @declaration a static method of `em25.BoxingAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Character.valueOf('\u0080');
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
