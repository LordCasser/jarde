// jarde: presentation of `C2$Inner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class C2$Inner extends java.lang.Object {
    private int tag;

    final C2 this$0;

    C2$Inner(C2 arg1, int arg2) {
        // @method <init>(LC2;I)V
        // @declaration a constructor of `C2$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.this$0 = arg1;
        super();
        this.tag = arg2;
        return;
    }

    int total() {
        // @method total()I
        // @declaration an instance method of `C2$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.tag + C2.access$000(this.this$0);
    }
}
