package dt29;

public class FieldCast {
	public static class A {
		public boolean publicField;
		boolean packagePrivateField;
		protected boolean protectedField;
		private boolean privateField;
	}

	public static class B extends A {
		public void self(boolean value) {
			((A) this).publicField = value;
			((A) this).protectedField = value;
			((A) this).packagePrivateField = value;
			((A) this).privateField = value;
		}
	}

	public static class C {
		public void set(B b, boolean value) {
			((A) b).publicField = value;
			((A) b).protectedField = value;
			((A) b).packagePrivateField = value;
			((A) b).privateField = value;
		}
	}

	private static class D {
		public <T extends B> void set(T t, boolean value) {
			((A) t).publicField = value;
			((A) t).protectedField = value;
			((A) t).packagePrivateField = value;
			((A) t).privateField = value;
		}
	}

	public static String run() {
		B b = new B();
		b.self(true);
		String first = bits(b);
		new C().set(b, false);
		String second = bits(b);
		new D().set(b, true);
		return first + ":" + second + ":" + bits(b);
	}

	private static String bits(A a) {
		return (a.publicField ? "1" : "0")
				+ (a.protectedField ? "1" : "0")
				+ (a.packagePrivateField ? "1" : "0")
				+ (a.privateField ? "1" : "0");
	}
}
