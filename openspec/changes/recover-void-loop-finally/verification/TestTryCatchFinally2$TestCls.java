// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally2$TestCls extends java.lang.Object {
    private jadx.core.clsp.ClspClass[] classes;

    public TestTryCatchFinally2$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test(java.io.OutputStream output) throws java.io.IOException {
        // @method test(Ljava/io/OutputStream;)V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.io.DataOutputStream out;
        out = new java.io.DataOutputStream(output);
        try {
            jadx.core.clsp.ClspClass[] local3;
            int local4;
            int local5;
            jadx.core.clsp.ClspClass cls;
            out.writeByte(1);
            out.writeInt(this.classes.length);
            local3 = this.classes;
            local4 = local3.length;
            local5 = 0;
            while (local5 < local4) {
                cls = local3[local5];
                this.writeString(out, (java.lang.String) cls.getName());
                local5 = local5 + 1;
            }
            local3 = this.classes;
            local4 = local3.length;
            local5 = 0;
            jadx.core.dex.instructions.args.ArgType[] local8;
            int local9;
            int local10;
            while (local5 < local4) {
                cls = local3[local5];
                jadx.core.dex.instructions.args.ArgType[] parents = cls.getParents();
                out.writeByte(parents.length);
                local8 = parents;
                local9 = local8.length;
                for (local10 = 0; local10 < local9; local10 = local10 + 1) {
                    jadx.core.dex.instructions.args.ArgType parent = local8[local10];
                    out.writeInt(parent.getObject().hashCode());
                }
                local5 = local5 + 1;
            }
        } finally {
            out.close();
        }
    }

    private void writeString(java.io.DataOutputStream out, java.lang.String name) {
        // @method writeString(Ljava/io/DataOutputStream;Ljava/lang/String;)V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }
}
