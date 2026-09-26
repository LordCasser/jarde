public class AnonymousInterfaceBasic {
	static I make() {
		return new I() {
			@Override
			public int value() {
				return 7;
			}
		};
	}

	public static void main(String[] args) {
		System.out.println(make().value());
	}
}
