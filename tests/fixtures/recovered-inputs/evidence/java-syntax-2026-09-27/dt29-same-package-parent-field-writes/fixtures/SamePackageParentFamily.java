package dt29;

public class SamePackageParentFamily {
	public static class A {
		public boolean publicField;
		protected boolean protectedField;
		boolean packagePrivateField;
		private boolean privateField;
		public static boolean staticField;
		int descriptorControl;
	}

	public static class B extends A {
		protected boolean protectedField;
		boolean packagePrivateField;

		public void set(boolean value, boolean privateValue) {
			((A) this).publicField = value;
			((A) this).protectedField = value;
			((A) this).packagePrivateField = value;
			((A) this).privateField = privateValue;
		}
	}
}
