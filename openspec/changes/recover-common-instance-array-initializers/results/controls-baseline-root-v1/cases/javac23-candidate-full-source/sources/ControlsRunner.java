import java.util.Arrays;

public class ControlsRunner {
	private static void require(boolean condition, String message) {
		if (!condition) {
			throw new AssertionError(message);
		}
	}

	private static void trace(String actual, String expected, String label) {
		require(expected.equals(actual), label + " trace: " + actual);
	}

	private static void bytes(byte[] actual, byte... expected) {
		require(Arrays.equals(actual, expected), "array: " + Arrays.toString(actual));
	}

	private static void nullBytes(byte[] actual, String label) {
		require(actual == null, label + " expected null array");
	}

	private static void commonConstructors() {
		CommonDirectSuperByteArray.trace = "";
		CommonDirectSuperByteArray c0 = new CommonDirectSuperByteArray();
		bytes(c0.bytes, (byte) 10, (byte) 20);
		trace(CommonDirectSuperByteArray.trace, "eval:10;eval:20;body:noarg;", "common noarg 0");
		CommonDirectSuperByteArray.trace = "";
		CommonDirectSuperByteArray c1 = new CommonDirectSuperByteArray();
		bytes(c1.bytes, (byte) 10, (byte) 20);
		trace(CommonDirectSuperByteArray.trace, "eval:10;eval:20;body:noarg;", "common noarg 1");
		require(c0.bytes != c1.bytes, "common noarg arrays are fresh");

		CommonDirectSuperByteArray.trace = "";
		CommonDirectSuperByteArray c2 = new CommonDirectSuperByteArray(4);
		bytes(c2.bytes, (byte) 10, (byte) 20);
		trace(CommonDirectSuperByteArray.trace, "eval:10;eval:20;body:int:4;", "common int 0");
		CommonDirectSuperByteArray.trace = "";
		CommonDirectSuperByteArray c3 = new CommonDirectSuperByteArray(5);
		bytes(c3.bytes, (byte) 10, (byte) 20);
		trace(CommonDirectSuperByteArray.trace, "eval:10;eval:20;body:int:5;", "common int 1");
		require(c2.bytes != c3.bytes, "common int arrays are fresh");
		System.out.println("base-common=" + Arrays.toString(c0.bytes) + ";" + Arrays.toString(c2.bytes));
	}

	private static void delegatingConstructors() {
		ThisDelegatingByteArray.trace = "";
		ThisDelegatingByteArray d0 = new ThisDelegatingByteArray();
		bytes(d0.bytes, (byte) 21, (byte) 22);
		trace(ThisDelegatingByteArray.trace, "eval:21;eval:22;body:target:7;body:delegate;", "delegate noarg 0");
		ThisDelegatingByteArray.trace = "";
		ThisDelegatingByteArray d1 = new ThisDelegatingByteArray();
		bytes(d1.bytes, (byte) 21, (byte) 22);
		trace(ThisDelegatingByteArray.trace, "eval:21;eval:22;body:target:7;body:delegate;", "delegate noarg 1");
		require(d0.bytes != d1.bytes, "delegating noarg arrays are fresh");

		ThisDelegatingByteArray.trace = "";
		ThisDelegatingByteArray d2 = new ThisDelegatingByteArray(8);
		bytes(d2.bytes, (byte) 21, (byte) 22);
		trace(ThisDelegatingByteArray.trace, "eval:21;eval:22;body:target:8;", "delegate int 0");
		ThisDelegatingByteArray.trace = "";
		ThisDelegatingByteArray d3 = new ThisDelegatingByteArray(9);
		bytes(d3.bytes, (byte) 21, (byte) 22);
		trace(ThisDelegatingByteArray.trace, "eval:21;eval:22;body:target:9;", "delegate int 1");
		require(d2.bytes != d3.bytes, "delegating int arrays are fresh");
		System.out.println("base-delegating=" + Arrays.toString(d0.bytes) + ";" + Arrays.toString(d2.bytes));
	}

	private static void differentRhsConstructors() {
		DifferentRhsByteArray.trace = "";
		DifferentRhsByteArray r0 = new DifferentRhsByteArray();
		bytes(r0.bytes, (byte) 31);
		trace(DifferentRhsByteArray.trace, "eval:31;body:noarg;", "different noarg 0");
		DifferentRhsByteArray.trace = "";
		DifferentRhsByteArray r1 = new DifferentRhsByteArray();
		bytes(r1.bytes, (byte) 31);
		trace(DifferentRhsByteArray.trace, "eval:31;body:noarg;", "different noarg 1");
		require(r0.bytes != r1.bytes, "different noarg arrays are fresh");

		DifferentRhsByteArray.trace = "";
		DifferentRhsByteArray r2 = new DifferentRhsByteArray(5);
		bytes(r2.bytes, (byte) 32);
		trace(DifferentRhsByteArray.trace, "eval:32;body:int:5;", "different int 0");
		DifferentRhsByteArray.trace = "";
		DifferentRhsByteArray r3 = new DifferentRhsByteArray(6);
		bytes(r3.bytes, (byte) 32);
		trace(DifferentRhsByteArray.trace, "eval:32;body:int:6;", "different int 1");
		require(r2.bytes != r3.bytes, "different int arrays are fresh");
		System.out.println("base-different=" + Arrays.toString(r0.bytes) + ";" + Arrays.toString(r2.bytes));
	}

	private static void finalLiteralConstructors() {
		FinalLiteralTwoArrays.trace = "";
		FinalLiteralTwoArrays f0 = new FinalLiteralTwoArrays();
		bytes(f0.first, (byte) 1, (byte) 2);
		bytes(f0.second, (byte) 3, (byte) 4);
		trace(FinalLiteralTwoArrays.trace, "body:noarg;", "final literal noarg 0");
		FinalLiteralTwoArrays.trace = "";
		FinalLiteralTwoArrays f1 = new FinalLiteralTwoArrays();
		bytes(f1.first, (byte) 1, (byte) 2);
		bytes(f1.second, (byte) 3, (byte) 4);
		trace(FinalLiteralTwoArrays.trace, "body:noarg;", "final literal noarg 1");
		require(f0.first != f1.first && f0.second != f1.second, "final noarg arrays are fresh");

		FinalLiteralTwoArrays.trace = "";
		FinalLiteralTwoArrays f2 = new FinalLiteralTwoArrays(2);
		bytes(f2.first, (byte) 1, (byte) 2);
		bytes(f2.second, (byte) 3, (byte) 4);
		trace(FinalLiteralTwoArrays.trace, "body:int:2;", "final literal int 0");
		FinalLiteralTwoArrays.trace = "";
		FinalLiteralTwoArrays f3 = new FinalLiteralTwoArrays(3);
		bytes(f3.first, (byte) 1, (byte) 2);
		bytes(f3.second, (byte) 3, (byte) 4);
		trace(FinalLiteralTwoArrays.trace, "body:int:3;", "final literal int 1");
		require(f2.first != f3.first && f2.second != f3.second, "final int arrays are fresh");
		System.out.println("final-literal=" + Arrays.toString(f0.first) + ";" + Arrays.toString(f0.second));
	}

	private static void missingWrite() {
		MissingWriteByteArray.trace = "";
		MissingWriteByteArray m0 = new MissingWriteByteArray();
		bytes(m0.bytes, (byte) 41);
		trace(MissingWriteByteArray.trace, "eval:41;body:write;", "missing write ctor 0");
		MissingWriteByteArray.trace = "";
		MissingWriteByteArray m1 = new MissingWriteByteArray();
		bytes(m1.bytes, (byte) 41);
		trace(MissingWriteByteArray.trace, "eval:41;body:write;", "missing write ctor 1");
		require(m0.bytes != m1.bytes, "missing-write path arrays are fresh");
		MissingWriteByteArray.trace = "";
		MissingWriteByteArray omitted0 = new MissingWriteByteArray(7);
		nullBytes(omitted0.bytes, "missing path 0");
		trace(MissingWriteByteArray.trace, "body:omit:7;", "missing path 0");
		MissingWriteByteArray.trace = "";
		MissingWriteByteArray omitted1 = new MissingWriteByteArray(8);
		nullBytes(omitted1.bytes, "missing path 1");
		trace(MissingWriteByteArray.trace, "body:omit:8;", "missing path 1");
		System.out.println("missing-write=" + Arrays.toString(m0.bytes) + ";omitted=null");
	}

	private static void duplicateWrite() {
		DuplicateWriteByteArray.trace = "";
		DuplicateWriteByteArray x0 = new DuplicateWriteByteArray();
		bytes(x0.bytes, (byte) 52);
		trace(DuplicateWriteByteArray.trace, "eval:51;eval:52;body:noarg;", "duplicate noarg 0");
		DuplicateWriteByteArray.trace = "";
		DuplicateWriteByteArray x1 = new DuplicateWriteByteArray();
		bytes(x1.bytes, (byte) 52);
		trace(DuplicateWriteByteArray.trace, "eval:51;eval:52;body:noarg;", "duplicate noarg 1");
		require(x0.bytes != x1.bytes, "duplicate noarg final arrays are fresh");
		DuplicateWriteByteArray.trace = "";
		DuplicateWriteByteArray y0 = new DuplicateWriteByteArray(9);
		bytes(y0.bytes, (byte) 51);
		trace(DuplicateWriteByteArray.trace, "eval:51;body:int:9;", "duplicate int 0");
		DuplicateWriteByteArray.trace = "";
		DuplicateWriteByteArray y1 = new DuplicateWriteByteArray(10);
		bytes(y1.bytes, (byte) 51);
		trace(DuplicateWriteByteArray.trace, "eval:51;body:int:10;", "duplicate int 1");
		require(y0.bytes != y1.bytes, "duplicate int arrays are fresh");
		System.out.println("duplicate=" + Arrays.toString(x0.bytes) + ";" + Arrays.toString(y0.bytes));
	}

	private static void interveningEffect() {
		InterveningEffectByteArray.trace = "";
		InterveningEffectByteArray g0 = new InterveningEffectByteArray();
		bytes(g0.bytes, (byte) 61);
		trace(InterveningEffectByteArray.trace, "gap;eval:61;body:noarg;", "gap noarg 0");
		InterveningEffectByteArray.trace = "";
		InterveningEffectByteArray g1 = new InterveningEffectByteArray();
		bytes(g1.bytes, (byte) 61);
		trace(InterveningEffectByteArray.trace, "gap;eval:61;body:noarg;", "gap noarg 1");
		require(g0.bytes != g1.bytes, "gap noarg arrays are fresh");
		InterveningEffectByteArray.trace = "";
		InterveningEffectByteArray h0 = new InterveningEffectByteArray(11);
		bytes(h0.bytes, (byte) 61);
		trace(InterveningEffectByteArray.trace, "eval:61;body:int:11;", "gap int 0");
		InterveningEffectByteArray.trace = "";
		InterveningEffectByteArray h1 = new InterveningEffectByteArray(12);
		bytes(h1.bytes, (byte) 61);
		trace(InterveningEffectByteArray.trace, "eval:61;body:int:12;", "gap int 1");
		require(h0.bytes != h1.bytes, "gap int arrays are fresh");
		System.out.println("intervening=" + Arrays.toString(g0.bytes) + ";" + Arrays.toString(h0.bytes));
	}

	private static void parameterRhs() {
		ParameterRhsByteArray.trace = "";
		ParameterRhsByteArray p0 = new ParameterRhsByteArray();
		bytes(p0.bytes, (byte) 71);
		trace(ParameterRhsByteArray.trace, "eval:71;body:noarg;", "parameter noarg 0");
		ParameterRhsByteArray.trace = "";
		ParameterRhsByteArray p1 = new ParameterRhsByteArray();
		bytes(p1.bytes, (byte) 71);
		trace(ParameterRhsByteArray.trace, "eval:71;body:noarg;", "parameter noarg 1");
		require(p0.bytes != p1.bytes, "parameter noarg arrays are fresh");
		ParameterRhsByteArray.trace = "";
		ParameterRhsByteArray q0 = new ParameterRhsByteArray(72);
		bytes(q0.bytes, (byte) 72);
		trace(ParameterRhsByteArray.trace, "eval:72;body:int:72;", "parameter int 0");
		ParameterRhsByteArray.trace = "";
		ParameterRhsByteArray q1 = new ParameterRhsByteArray(73);
		bytes(q1.bytes, (byte) 73);
		trace(ParameterRhsByteArray.trace, "eval:73;body:int:73;", "parameter int 1");
		require(q0.bytes != q1.bytes, "parameter int arrays are fresh");
		System.out.println("parameter-rhs=" + Arrays.toString(p0.bytes) + ";"
				+ Arrays.toString(q0.bytes) + ";" + Arrays.toString(q1.bytes));
	}

	private static void reverseFieldOrder() {
		ReverseFieldOrderByteArray.trace = "";
		ReverseFieldOrderByteArray v0 = new ReverseFieldOrderByteArray();
		bytes(v0.first, (byte) 81);
		bytes(v0.second, (byte) 82);
		trace(ReverseFieldOrderByteArray.trace, "eval:82;eval:81;body:noarg;", "reverse noarg 0");
		ReverseFieldOrderByteArray.trace = "";
		ReverseFieldOrderByteArray v1 = new ReverseFieldOrderByteArray();
		bytes(v1.first, (byte) 81);
		bytes(v1.second, (byte) 82);
		trace(ReverseFieldOrderByteArray.trace, "eval:82;eval:81;body:noarg;", "reverse noarg 1");
		require(v0.first != v1.first && v0.second != v1.second, "reverse noarg arrays are fresh");
		ReverseFieldOrderByteArray.trace = "";
		ReverseFieldOrderByteArray w0 = new ReverseFieldOrderByteArray(14);
		bytes(w0.first, (byte) 81);
		bytes(w0.second, (byte) 82);
		trace(ReverseFieldOrderByteArray.trace, "eval:82;eval:81;body:int:14;", "reverse int 0");
		ReverseFieldOrderByteArray.trace = "";
		ReverseFieldOrderByteArray w1 = new ReverseFieldOrderByteArray(15);
		bytes(w1.first, (byte) 81);
		bytes(w1.second, (byte) 82);
		trace(ReverseFieldOrderByteArray.trace, "eval:82;eval:81;body:int:15;", "reverse int 1");
		require(w0.first != w1.first && w0.second != w1.second, "reverse int arrays are fresh");
		System.out.println("reverse-fields=" + Arrays.toString(v0.first) + ";" + Arrays.toString(v0.second));
	}

	private static void handlerNoarg() {
		HandlerArrayByteArray.fail = false;
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e0 = new HandlerArrayByteArray();
		bytes(e0.bytes, (byte) 91);
		trace(HandlerArrayByteArray.trace, "eval:91;body:noarg;", "handler noarg success 0");
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e1 = new HandlerArrayByteArray();
		bytes(e1.bytes, (byte) 91);
		trace(HandlerArrayByteArray.trace, "eval:91;body:noarg;", "handler noarg success 1");
		require(e0.bytes != e1.bytes, "handler noarg arrays are fresh");
		HandlerArrayByteArray.fail = true;
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e2 = new HandlerArrayByteArray();
		nullBytes(e2.bytes, "handler noarg caught 0");
		trace(HandlerArrayByteArray.trace, "eval:91;caught;body:noarg;", "handler noarg caught 0");
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e3 = new HandlerArrayByteArray();
		nullBytes(e3.bytes, "handler noarg caught 1");
		trace(HandlerArrayByteArray.trace, "eval:91;caught;body:noarg;", "handler noarg caught 1");
		HandlerArrayByteArray.fail = false;
		System.out.println("handler-noarg=" + Arrays.toString(e0.bytes) + ";caught=null");
	}

	private static void handlerInt() {
		HandlerArrayByteArray.fail = false;
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e0 = new HandlerArrayByteArray(16);
		bytes(e0.bytes, (byte) 91);
		trace(HandlerArrayByteArray.trace, "eval:91;body:int:16;", "handler int success 0");
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e1 = new HandlerArrayByteArray(17);
		bytes(e1.bytes, (byte) 91);
		trace(HandlerArrayByteArray.trace, "eval:91;body:int:17;", "handler int success 1");
		require(e0.bytes != e1.bytes, "handler int arrays are fresh");
		HandlerArrayByteArray.fail = true;
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e2 = new HandlerArrayByteArray(18);
		nullBytes(e2.bytes, "handler int caught 0");
		trace(HandlerArrayByteArray.trace, "eval:91;caught;body:int:18;", "handler int caught 0");
		HandlerArrayByteArray.trace = "";
		HandlerArrayByteArray e3 = new HandlerArrayByteArray(19);
		nullBytes(e3.bytes, "handler int caught 1");
		trace(HandlerArrayByteArray.trace, "eval:91;caught;body:int:19;", "handler int caught 1");
		HandlerArrayByteArray.fail = false;
		System.out.println("handler-int=" + Arrays.toString(e0.bytes) + ";caught=null");
	}

	public static void main(String[] args) {
		commonConstructors();
		delegatingConstructors();
		differentRhsConstructors();
		finalLiteralConstructors();
		missingWrite();
		duplicateWrite();
		interveningEffect();
		parameterRhs();
		reverseFieldOrder();
		handlerNoarg();
		handlerInt();
	}
}
