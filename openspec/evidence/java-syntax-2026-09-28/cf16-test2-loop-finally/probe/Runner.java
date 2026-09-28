package jadx.tests.integration.trycatch;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.util.Arrays;
import jadx.core.clsp.ClspClass;
import jadx.core.dex.instructions.args.ArgType;

public final class Runner {
	private static final class Sink extends OutputStream {
		final ByteArrayOutputStream bytes = new ByteArrayOutputStream();
		final int failWrite;
		final boolean failClose;
		int writes;
		int closes;
		Sink(int failWrite, boolean failClose) { this.failWrite = failWrite; this.failClose = failClose; }
		@Override public void write(int value) throws IOException {
			writes++;
			if (writes == failWrite) throw new IOException("write-" + writes);
			bytes.write(value);
		}
		@Override public void write(byte[] data, int off, int len) throws IOException {
			writes++;
			if (writes == failWrite) throw new IOException("write-" + writes);
			bytes.write(data, off, len);
		}
		@Override public void close() throws IOException {
			closes++;
			if (failClose) throw new IOException("close");
		}
	}

	private static ClspClass[] values() {
		return new ClspClass[] {
			new ClspClass("A", new ArgType("p.A"), new ArgType("p.B")),
			new ClspClass("B", new ArgType("p.C"))
		};
	}

	private static void run(String label, int failWrite, boolean failClose, int count) throws Exception {
		ArgType.totalReads = 0;
		Class<?> targetClass = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls");
		Object target = targetClass.getConstructor().newInstance();
		ClspClass[] classes = Arrays.copyOf(values(), count);
		Field field = targetClass.getDeclaredField("classes");
		field.setAccessible(true);
		field.set(target, classes);
		Sink sink = new Sink(failWrite, failClose);
		String outcome = "ok";
		try { targetClass.getMethod("test", OutputStream.class).invoke(target, sink); }
		catch (InvocationTargetException wrapped) {
			Throwable ex = wrapped.getCause();
			outcome = ex.getClass().getSimpleName() + ":" + ex.getMessage();
		}
		int names = 0, parentReads = 0, objectReads = 0;
		for (ClspClass cls : classes) {
			names += cls.nameReads;
			parentReads += cls.parentReads;
		}
		objectReads = ArgType.totalReads;
		System.out.printf("%s outcome=%s writes=%d closes=%d bytes=%s nameReads=%d parentReads=%d objectReads=%d%n",
			label, outcome, sink.writes, sink.closes, hex(sink.bytes.toByteArray()), names, parentReads, objectReads);
	}

	private static String hex(byte[] data) {
		StringBuilder result = new StringBuilder();
		for (byte b : data) result.append(String.format("%02x", b & 0xff));
		return result.toString();
	}

	public static void main(String[] args) throws Exception {
		run("empty", 0, false, 0);
		run("two-classes-three-parents", 0, false, 2);
		run("write-failure-first-write", 1, false, 2);
		run("write-failure-parent-write", 4, false, 2);
		run("close-failure", 0, true, 2);
		run("write-and-close-failure", 1, true, 2);
	}
}
