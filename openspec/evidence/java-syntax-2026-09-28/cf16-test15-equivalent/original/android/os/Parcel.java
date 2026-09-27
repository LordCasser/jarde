package android.os;

public final class Parcel {
	public static final StringBuilder EVENTS = new StringBuilder();
	public static boolean failRead;
	public static boolean failRecycleOnce;

	private final String name;

	public Parcel(String name) {
		this.name = name;
	}

	public static Parcel obtain() {
		EVENTS.append("obtain;");
		return new Parcel("out");
	}

	public void readException() {
		EVENTS.append("read;");
		if (failRead) {
			throw new IllegalStateException("read");
		}
	}

	public void recycle() {
		EVENTS.append(name).append(".recycle;");
		if ("in".equals(name) && failRecycleOnce) {
			failRecycleOnce = false;
			throw new IllegalStateException("recycle");
		}
	}

	public String name() {
		return name;
	}
}
