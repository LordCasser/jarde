import java.util.Arrays;
import java.util.Collections;

public class Runner {
	public static void main(String[] args) {
		System.out.println("null=" + VariablePostfixLoop.countEmpty(null));
		System.out.println("empty=" + VariablePostfixLoop.countEmpty(Collections.<String>emptyList()));
		System.out.println("mixed=" + VariablePostfixLoop.countEmpty(Arrays.asList("", "x", "")));
		System.out.println("repeat=" + VariablePostfixLoop.countEmpty(Arrays.asList("x", "")));
		try {
			VariablePostfixLoop.countEmpty(Arrays.asList("", null));
			System.out.println("null-element=missing-npe");
		} catch (NullPointerException expected) {
			System.out.println("null-element=NullPointerException");
		}
	}
}
