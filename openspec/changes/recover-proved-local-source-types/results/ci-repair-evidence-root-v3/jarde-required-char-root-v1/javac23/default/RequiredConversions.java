// jarde: presentation of `RequiredConversions` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class RequiredConversions extends java.lang.Object {
    static int field;

    public RequiredConversions() {
        // @method <init>()V
        // @declaration a constructor of `RequiredConversions`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String castPart(char arg0) {
        // @method castPart(C)Ljava/lang/String;
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "" + (int) arg0 + "!";
    }

    public static java.lang.String castPartLast(java.lang.String arg0, char arg1) {
        // @method castPartLast(Ljava/lang/String;C)Ljava/lang/String;
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + (int) arg1;
    }

    public static java.lang.String intPart(int arg0) {
        // @method intPart(I)Ljava/lang/String;
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "" + arg0 + "!";
    }

    public static int widen(int arg0) {
        // @method widen(I)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static int argued(char arg0) {
        // @method argued(C)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return widen((int) arg0);
    }

    public static int arguedByte(byte arg0) {
        // @method arguedByte(B)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return widen((int) arg0);
    }

    public static int returned(char arg0) {
        // @method returned(C)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static int returnedShort(short arg0) {
        // @method returnedShort(S)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static char kept(char arg0) {
        // @method kept(C)C
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static int declared(char arg0) {
        // @method declared(C)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char local1 = arg0;
        return local1;
    }

    public static int assigned(char arg0) {
        // @method assigned(C)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char local1 = '\u0000';
        local1 = arg0;
        return local1;
    }

    public static int written(char arg0) {
        // @method written(C)I
        // @declaration a static method of `RequiredConversions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        RequiredConversions.field = arg0;
        return RequiredConversions.field;
    }
}
