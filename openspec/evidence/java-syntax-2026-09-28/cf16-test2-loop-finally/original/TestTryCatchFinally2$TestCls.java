package jadx.tests.integration.trycatch;

import java.io.DataOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import jadx.core.clsp.ClspClass;
import jadx.core.dex.instructions.args.ArgType;

public class TestTryCatchFinally2$TestCls {
	private ClspClass[] classes;

	public void test(OutputStream output) throws IOException {
		DataOutputStream out = new DataOutputStream(output);
		try {
			out.writeByte(1);
			out.writeInt(classes.length);
			for (ClspClass cls : classes) {
				writeString(out, cls.getName());
			}
			for (ClspClass cls : classes) {
				ArgType[] parents = cls.getParents();
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
