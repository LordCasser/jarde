import java.util.Arrays;

public class Runner {
	public static void main(String[] args) {
		ArrayFieldLiteral first = new ArrayFieldLiteral();
		ArrayFieldLiteral second = new ArrayFieldLiteral();
		System.out.println("static=" + Arrays.toString(ArrayFieldLiteral.a));
		System.out.println("first=" + Arrays.toString(first.b));
		System.out.println("second=" + Arrays.toString(second.b));
		System.out.println("fresh=" + (first.b != second.b));
	}
}
