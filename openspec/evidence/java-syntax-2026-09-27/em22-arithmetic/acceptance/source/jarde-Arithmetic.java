// jarde: presentation of `em22/Arithmetic` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em22;

public class Arithmetic extends java.lang.Object {
    private int calls;

    public Arithmetic() {
        // @method <init>()V
        // @declaration a constructor of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int multiply(int arg1) {
        // @method multiply(I)I
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return (arg1 + 2) * 3;
    }

    public int sum(int arg1, int arg2, int arg3) {
        // @method sum(III)I
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 + arg2 + arg3;
    }

    public int subtract(int arg1, int arg2, int arg3) {
        // @method subtract(III)I
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 - (arg2 - arg3);
    }

    public int divide(int arg1, int arg2, int arg3) {
        // @method divide(III)I
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 / (arg2 / arg3);
    }

    public boolean or(boolean arg1, boolean arg2, boolean arg3) {
        // @method or(ZZZ)Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 | arg2 | arg3;
    }

    public boolean and(boolean arg1, boolean arg2, boolean arg3) {
        // @method and(ZZZ)Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 & arg2 & arg3;
    }

    public int notInt(int arg1) {
        // @method notInt(I)I
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 ^ -1;
    }

    public long notLong(long arg1) {
        // @method notLong(J)J
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 ^ -1L;
    }

    public boolean flip(boolean arg1) {
        // @method flip(Z)Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 ^ true;
    }

    public boolean flipCall() {
        // @method flipCall()Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.left() ^ true;
    }

    public boolean sameCall() {
        // @method sameCall()Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.left() ^ false;
    }

    private boolean left() {
        // @method left()Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.calls += 1;
        return true;
    }

    private boolean right() {
        // @method right()Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.calls += 1;
        return false;
    }

    public boolean eagerOr() {
        // @method eagerOr()Z
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.left() | this.right();
    }

    public int calls() {
        // @method calls()I
        // @declaration an instance method of `em22.Arithmetic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.calls;
    }
}
