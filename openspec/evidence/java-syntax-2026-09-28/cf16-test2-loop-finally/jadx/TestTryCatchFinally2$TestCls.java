package jadx.tests.integration.trycatch;

import jadx.core.clsp.ClspClass;
import jadx.core.dex.instructions.args.ArgType;
import java.io.DataOutputStream;
import java.io.IOException;
import java.io.OutputStream;

/* JADX INFO: loaded from: TestTryCatchFinally2$TestCls.class */
public class TestTryCatchFinally2$TestCls {
    private ClspClass[] classes;

    public void test(OutputStream output) throws IOException {
        DataOutputStream out = new DataOutputStream(output);
        try {
            out.writeByte(1);
            out.writeInt(this.classes.length);
            for (ClspClass cls : this.classes) {
                writeString(out, cls.getName());
            }
            for (ClspClass cls2 : this.classes) {
                ArgType[] parents = cls2.getParents();
                out.writeByte(parents.length);
                for (ArgType parent : parents) {
                    out.writeInt(parent.getObject().hashCode());
                }
            }
        } finally {
            out.close();
        }
    }

    private void writeString(DataOutputStream out, String name) {
    }
}
