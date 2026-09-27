// jarde: presentation of `p/Nested$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

class Nested$1 extends java.lang.Object implements p.Factory {
    Nested$1() {
        // @method <init>()V
        // @declaration a constructor of `p.Nested$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public p.Action make() {
        // @method make()Lp/Action;
        // @declaration an instance method of `p.Nested$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new p.Nested$1$1(this);
    }
}
