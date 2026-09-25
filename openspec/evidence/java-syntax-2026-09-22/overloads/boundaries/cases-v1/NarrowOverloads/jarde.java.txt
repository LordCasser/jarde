// jarde: presentation of `NarrowOverloads` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NarrowOverloads extends java.lang.Object {
    public NarrowOverloads() {
        // @method <init>()V
        // @declaration a constructor of `NarrowOverloads`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int onlyByte(byte arg0) {
        // @method onlyByte(B)I
        // @declaration a static method of `NarrowOverloads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 1;
    }

    public static int onlyByte(int arg0) {
        // @method onlyByte(I)I
        // @declaration a static method of `NarrowOverloads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 2;
    }

    public static int onlyShort(short arg0) {
        // @method onlyShort(S)I
        // @declaration a static method of `NarrowOverloads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 3;
    }

    public static int onlyShort(int arg0) {
        // @method onlyShort(I)I
        // @declaration a static method of `NarrowOverloads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 4;
    }

    public static int runByte() {
        // @method runByte()I
        // @declaration a static method of `NarrowOverloads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return onlyByte(3);
    }

    public static int runShort() {
        // @method runShort()I
        // @declaration a static method of `NarrowOverloads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return onlyShort(3);
    }
}
