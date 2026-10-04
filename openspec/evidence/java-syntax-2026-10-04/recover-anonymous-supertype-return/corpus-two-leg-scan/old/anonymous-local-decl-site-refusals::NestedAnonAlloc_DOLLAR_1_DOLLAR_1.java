// jarde: presentation of `NestedAnonAlloc$1$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class NestedAnonAlloc$1$1 extends java.lang.Object implements java.lang.Runnable {
    final NestedAnonAlloc$1 this$0;

    NestedAnonAlloc$1$1(NestedAnonAlloc$1 arg1) {
        // @method <init>(LNestedAnonAlloc$1;)V
        // @declaration a constructor of `NestedAnonAlloc$1$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.this$0 = arg1;
        return;
    }

    public void run() {
        // @method run()V
        // @declaration an instance method of `NestedAnonAlloc$1$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        NestedAnonAlloc.event("helper-ran");
        return;
    }
}
