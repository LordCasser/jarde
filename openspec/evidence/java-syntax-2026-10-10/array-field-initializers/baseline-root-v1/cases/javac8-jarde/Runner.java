import java.util.Arrays;

public class Runner {
	public static void main(String[] args) {
		System.out.println("static=" + ArrayFieldInitializers.trace + ":"
				+ ArrayFieldInitializers.before + ":"
				+ Arrays.toString(ArrayFieldInitializers.a) + ":"
				+ ArrayFieldInitializers.after);

		ArrayFieldInitializers first = new ArrayFieldInitializers();
		System.out.println("first=" + ArrayFieldInitializers.trace + ":"
				+ Arrays.toString(first.b));

		ArrayFieldInitializers second = new ArrayFieldInitializers();
		System.out.println("second=" + ArrayFieldInitializers.trace + ":"
				+ Arrays.toString(second.b));
	}
}
