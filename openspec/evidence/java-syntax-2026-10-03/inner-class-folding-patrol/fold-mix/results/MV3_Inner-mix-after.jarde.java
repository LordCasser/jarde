// jarde: presentation of `MV3$Inner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class MV3$Inner extends java.lang.Object {
    int tag;

    final MV3 this$0;

    MV3$Inner(MV3 arg1, int arg2) {
        // @method <init>(LMV3;I)V
        // @declaration a constructor of `MV3$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.this$0 = arg1;
        this.tag = arg2;
        return;
    }

    int total() {
        // @method total()I
        // @declaration an instance method of `MV3$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.tag + MV3.access$000(this.this$0);
    }
}
