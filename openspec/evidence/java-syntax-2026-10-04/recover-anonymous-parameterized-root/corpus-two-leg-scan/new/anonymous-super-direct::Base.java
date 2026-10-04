// jarde: presentation of `Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class Base extends java.lang.Object {
    private final int left;

    private final int right;

    Base(int arg1, int arg2) {
        // @method <init>(II)V
        // @declaration a constructor of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.left = arg1;
        this.right = arg2;
        return;
    }

    Base(int arg1, long arg2) {
        // @method <init>(IJ)V
        // @declaration a constructor of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, (int) arg2);
        return;
    }

    int sum() {
        // @method sum()I
        // @declaration an instance method of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.left * 2 - this.right;
    }
}
