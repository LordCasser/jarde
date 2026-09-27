package dt29;

public class PrivateFieldFamily {
	public static class A {
		public boolean visible;
		private boolean hidden;
	}

	public static class B extends A {
		public boolean visible;

		public void set(boolean visible, boolean hidden) {
			((A) this).visible = visible;
			((A) this).hidden = hidden;
		}
	}
}
