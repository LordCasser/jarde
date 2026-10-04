public class AnonymousSuperArgs {
	private static final StringBuilder EVENTS = new StringBuilder();

	static void event(String event) {
		if (EVENTS.length() != 0) {
			EVENTS.append('|');
		}
		EVENTS.append(event);
	}

	private static String text(String name, String value) {
		event("arg:" + name);
		return value;
	}

	private static int number(String name, int value) {
		event("arg:" + name);
		return value;
	}

	public static void main(String[] args) {
		final String captured = text("capture", "captured");
		Base instance = new Base(text("super-label", "explicit"), number("super-value", 17)) {
			@Override
			String render() {
				event("body:" + captured);
				return super.render() + ":" + captured;
			}
		};
		System.out.println(EVENTS);
		System.out.println(instance.render());
	}
}
