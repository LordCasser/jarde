class Base {
	private final String label;
	private final int value;

	Base(String label, int value) {
		this.label = label;
		this.value = value;
		AnonymousSuperArgs.event("base:" + label + ":" + value);
	}

	String render() {
		return label + ":" + value;
	}
}
