// jarde: presentation of `p/Nested$1$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

class Nested$1$1 extends java.lang.Object implements p.Action {
    final p.Nested$1 this$0;

    Nested$1$1(p.Nested$1 arg1) {
        // @method <init>(Lp/Nested$1;)V
        // @declaration a constructor of `p.Nested$1$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.this$0 = arg1;
        super();
        return;
    }

    public void run() {
        // @method run()V
        // @declaration an instance method of `p.Nested$1$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        p.Nested.trace = p.Nested.trace + 1;
        return;
    }
}
