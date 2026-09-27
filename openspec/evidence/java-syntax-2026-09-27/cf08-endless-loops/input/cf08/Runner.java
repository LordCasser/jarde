package cf08;

import java.io.File;

public final class Runner {
	public static void main(String[] args) {
		NotIndexedLoop subject = new NotIndexedLoop();
		print(subject.test(null));
		print(subject.test(new File[] {}));
		print(subject.test(new File[] { new File("a"), new File("f") }));
		print(subject.test(new File[] { new File("a") }));
	}

	private static void print(File file) {
		System.out.println(file == null ? "null" : file.getName());
	}
}
