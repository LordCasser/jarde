// jarde: presentation of `p/Child` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

class Child extends p.Base implements p.A, p.B {
    Child() {
        // @method <init>()V
        // @declaration a constructor of `p.Child`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int m() {
        // @method m()I
        // @declaration an instance method of `p.Child`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return p.A.super.m();
    }

    int call() {
        // @method call()I
        // @declaration an instance method of `p.Child`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return p.A.super.m();
    }
}
