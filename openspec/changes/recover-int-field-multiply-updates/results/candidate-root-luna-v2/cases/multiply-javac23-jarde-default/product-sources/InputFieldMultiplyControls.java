// jarde: presentation of `em23/InputFieldMultiplyControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em23;

public class InputFieldMultiplyControls extends java.lang.Object {
    public A a;

    public InputFieldMultiplyControls() {
        // @method <init>()V
        // @declaration a constructor of `em23.InputFieldMultiplyControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test1(int arg1) {
        // @method test1(I)V
        // @declaration an instance method of `em23.InputFieldMultiplyControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.a.f = this.a.f + arg1;
        return;
    }

    public void test2(int arg1) {
        // @method test2(I)V
        // @declaration an instance method of `em23.InputFieldMultiplyControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.a.f *= arg1;
        return;
    }

    public int multiplyDivide(int arg1) {
        // @method multiplyDivide(I)I
        // @declaration an instance method of `em23.InputFieldMultiplyControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.a.f *= 8 / arg1;
        return this.a.f;
    }

    private static class A extends java.lang.Object {
        int f;

        private A() {
            // @method <init>()V
            // @declaration a constructor of `em23.InputFieldMultiplyControls$A`, member flags 0x0002
            // recovered from bytecode; presentation is not claimed to compile
            super();
            this.f = 5;
            return;
        }
    }
}
