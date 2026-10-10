// jarde: presentation of `CommonNoClinitArrayInit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CommonNoClinitArrayInit extends ArrayFieldInitBase {
    static int trace;

    final byte[] first;

    final byte[] second;

    static byte mark(int arg0) {
        // @method mark(I)B
        // @declaration a static method of `CommonNoClinitArrayInit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        CommonNoClinitArrayInit.trace = CommonNoClinitArrayInit.trace * 31 + arg0;
        return (byte) arg0;
    }

    static byte run(int arg0) {
        // @method run(I)B
        // @declaration a static method of `CommonNoClinitArrayInit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return mark(arg0);
    }

    public CommonNoClinitArrayInit() {
        // @method <init>()V
        // @declaration a constructor of `CommonNoClinitArrayInit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(7);
        this.first = new byte[]{mark(11), mark(12)};
        this.second = new byte[]{run(21)};
        CommonNoClinitArrayInit.trace = CommonNoClinitArrayInit.trace * 31 + 91;
        return;
    }

    public CommonNoClinitArrayInit(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `CommonNoClinitArrayInit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1);
        this.first = new byte[]{mark(11), mark(12)};
        this.second = new byte[]{run(21)};
        CommonNoClinitArrayInit.trace = CommonNoClinitArrayInit.trace * 31 + arg1;
        return;
    }
}
